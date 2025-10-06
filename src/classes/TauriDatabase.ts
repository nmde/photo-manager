import type { Constructor } from '../types/Constructor';
import type { Entity } from './Entity';
import Database from '@tauri-apps/plugin-sql';
import { EventEmitter } from 'ee-ts';

export class TauriDatabase extends EventEmitter<{
  endQuery: () => void;
  startQuery: () => void;
  queryError: (error: string) => void;
}> {
  private connected = false;

  private database!: Database;

  private ensuredTables: string[] = [];

  public constructor(public path: string) {
    super();
  }

  /**
   * Deletes the given entity from the database.
   * @param entity - The entity to delete.
   */
  public async delete(entity: Entity) {
    const query = `DELETE FROM ${entity.tableName} WHERE Id='${entity.Id}'`;
    await this.execute(query);
  }

  /**
   * Deletes the entity matching the given parameters
   * @param From - The table to delete from.
   * @param options - The options to search by.
   */
  public async deleteWhere<T extends Entity>(From: Constructor<T>, options: Partial<T['data']>) {
    const dummy = new From();
    let query = `DELETE FROM ${dummy.tableName} WHERE `;
    for (const [key, value] of Object.entries(options)) {
      query += `${key}=${this.getCleanValue(value)}, `;
    }
    query = query.slice(0, Math.max(0, query.length - 2));
    await this.execute(query);
  }

  /**
   * Shortcut for safely executing a SQL query.
   * @param query - The query to execute.
   */
  public async execute(query: string) {
    console.log(query);
    this.emit('startQuery');
    try {
      const result = await (await this.getConnection()).execute(query);
      this.emit('endQuery');
      return result;
    } catch (error) {
      console.error(error);
      this.emit('queryError', (error as Error).message);
      this.emit('endQuery');
      return null;
    }
  }

  /**
   * Checks if the given entity exists in the databse.
   * @param entity - The entity to check for.
   */
  public async exists(entity: Entity) {
    return (
      (
        await this.select(
          `SELECT * FROM ${entity.tableName} WHERE ${
            entity.primaryKey
              ? `${entity.primaryKey}=${this.getCleanValue(entity.data[entity.primaryKey])}`
              : `Id='${entity.Id}'`
          }`,
        )
      ).length > 0
    );
  }

  /**
   * Inserts items into a table.
   * @param entity - The entity to insert.
   * @param autoUpdate - If the entity already exists, automatically update instead of inserting.
   */
  public async insert(entity: Entity, autoUpdate = true) {
    await this.ensureTable(entity);
    if (await this.exists(entity)) {
      if (autoUpdate) {
        await this.update(entity);
      } else {
        throw new Error(`Entity already exists: ${entity.tableName}/${entity.Id}`);
      }
    } else {
      let query = `INSERT INTO ${entity.tableName} (Id,`;
      let values = `'${entity.Id}', `;
      for (const name of Object.keys(entity.data)) {
        query += `${this.getCleanValue(name)}, `;
        values += `${this.getCleanValue(entity.data[name])}, `;
      }
      query = query.slice(0, Math.max(0, query.length - 2));
      values = values.slice(0, Math.max(0, values.length - 2));
      query += `) VALUES (${values})`;
      await this.execute(query);
    }
  }

  /**
   * Inserts multiple entities into the database.
   * @param entities - The entities to insert.
   * @param autoUpdate - If the entity already exists, automatically update instead of inserting.
   * @param skipExisting - If existing entities should silently be skipped.
   */
  public async insertAll(entities: Entity[], autoUpdate = true, skipExisting = false) {
    if (entities.length === 0) {
      return;
    }
    const table = entities[0]?.tableName;
    await this.ensureTable(entities[0]);
    const toInsert = [];
    const toUpdate = [];
    for (const entity of entities) {
      if (entity.tableName !== table) {
        throw new Error('Cannot insert multiple kinds of entities in the same call.');
      }
      if (await this.exists(entity)) {
        if (autoUpdate) {
          toUpdate.push(entity);
        } else if (!skipExisting) {
          throw new Error(`Entity already exists: ${entity.tableName}/${entity.Id}`);
        }
      } else {
        toInsert.push(entity);
      }
    }
    if (toInsert.length > 0) {
      let query = `INSERT INTO ${table} (Id,`;
      for (const name of Object.keys(toInsert[0].data)) {
        query += `${this.getCleanValue(name)}, `;
      }
      query = query.slice(0, Math.max(0, query.length - 2));
      let values = '';
      for (const entity of toInsert) {
        values += `('${entity.Id}', `;
        for (const name of Object.keys(entity.data)) {
          values += `${this.getCleanValue(entity.data[name])}, `;
        }
        values = values.slice(0, Math.max(0, values.length - 2)) + '), ';
      }
      query += `) VALUES ${values.slice(0, Math.max(0, values.length - 2))}`;
      await this.execute(query);
    }
    if (toUpdate.length > 0) {
      await this.updateAll(toUpdate);
    }
  }

  /**
   * Selects and returns all entries from the given table.
   * @param from
   */
  public async selectAll<T extends Entity>(From: Constructor<T>): Promise<T[]> {
    try {
      return (await this.select(`SELECT * FROM ${new From().tableName}`)).map(result => {
        const obj = new From(result);
        obj.Id = result.Id;
        return obj;
      });
    } catch {
      return [];
    }
  }

  /**
   * Selects entities that match the data provided.
   * @param entity - The type of entity to select.
   * @param options - The data to search for.
   * @returns Entities matching the
   */
  public async selectWhere<T extends Entity>(
    From: Constructor<T>,
    options: Partial<T['data']>,
  ): Promise<T[]> {
    const dummy = new From();
    let query = `SELECT * FROM ${dummy.tableName} WHERE `;
    for (const [key, value] of Object.entries(options)) {
      query += `${key}=${this.getCleanValue(value)}, `;
    }
    query = query.slice(0, Math.max(0, query.length - 2));
    return (await this.select(query)).map(result => {
      const obj = new From(result);
      obj.Id = result.Id;
      return obj;
    });
  }

  /**
   * Updates an entity.
   * @param entity - The entity to update.
   */
  public async update(entity: Entity) {
    await this.ensureTable(entity);
    let query = `UPDATE ${entity.tableName} SET `;
    for (const name of Object.keys(entity.data)) {
      query += `${name} = ${this.getCleanValue(entity.data[name])}, `;
    }
    await this.execute(
      query.slice(0, Math.max(0, query.length - 2)) +
      ' WHERE ' +
      (entity.primaryKey
          ? `${entity.primaryKey}=${this.getCleanValue(entity.data[entity.primaryKey])}`
          : `Id='${entity.Id}'`),
    );
  }

  /**
   * Updates all the given entities.
   * @param entities - The entities to update.
   */
  public async updateAll(entities: Entity[]) {
    // TODO
    for (const entity of entities) {
      await this.update(entity);
    }
  }

  /**
   * Ensures a table exists.
   * @param entity - The entity to model the table after.
   */
  private async ensureTable(entity: Entity) {
    if (!this.ensuredTables.includes(entity.tableName)) {
      let query = `CREATE TABLE IF NOT EXISTS ${entity.tableName} (Id TEXT ${
        entity.primaryKey ? ', ' : 'PRIMARY KEY, '
      }`;
      for (const [name, value] of Object.entries(entity.data)) {
        let str = `${name} `;
        if (typeof value === 'number' || typeof value === 'boolean') {
          str += 'INTEGER';
        } else if (typeof value === 'string' || value === null) {
          str += 'TEXT';
        } else {
          throw new Error(`Unhandled data type for ${name}!`);
        }
        query += str + name === entity.primaryKey ? ` PRIMARY KEY, ` : ', ';
      }
      query = query.slice(0, Math.max(0, query.length - 2)) + ')';
      await this.execute(query);
      this.ensuredTables.push(entity.tableName);
    }
  }

  /**
   * Gets a cleaned value.
   * @param value - The value to clean.
   */
  private getCleanValue(value: any) {
    if (value === null) {
      return '\'\'';
    }
    if (typeof value === 'string') {
      return `'${value.replace(/'/g, '\'\'')}'`;
    }
    if (typeof value === 'number') {
      return value.toString();
    }
    if (typeof value === 'boolean') {
      if (value) {
        return '1';
      }
      return '0';
    }
    throw new Error(`Unhandled data type for ${value}!`);
  }

  /**
   * Safely gets the local database connection.
   * @returns
   */
  private async getConnection() {
    if (!this.connected) {
      this.database = await Database.load(this.path);
    }
    return this.database;
  }

  /**
   * Performs a SELECT query.
   * @param query - The query to run.
   */
  private async select(query: string): Promise<any[]> {
    console.log(query);
    return await (await this.getConnection()).select(query);
  }
}
