import Database from 'tauri-plugin-sql-api';
import { Entity } from './Entity';
import { Constructor } from '../types/Constructor';

export class TauriDatabase {
  private connected = false;

  private database!: Database;

  private ensuredTables: string[] = [];

  public constructor(public path: string) {}

  /**
   * Deletes the given entity from the database.
   * @param entity - The entity to delete.
   */
  public async delete(entity: Entity) {
    const query = `DELETE FROM ${entity.tableName} WHERE Id='${entity.id}'`;
    await this.execute(query);
  }

  /**
   * Ensures a table exists.
   * @param entity - The entity to model the table after.
   */
  private async ensureTable(entity: Entity) {
    if (this.ensuredTables.indexOf(entity.tableName) < 0) {
      let query = `CREATE TABLE IF NOT EXISTS ${entity.tableName} (Id TEXT PRIMARY KEY,`;
      Object.entries(entity.data).forEach(([name, value]) => {
        let str = `${name} `;
        if (typeof value === 'number') {
          str += 'INTEGER';
        } else if (typeof value === 'string') {
          str += 'TEXT';
        } else {
          throw new Error(`Unhandled data type for ${name}!`);
        }
        query += `${str}, `;
      });
      query = query.substring(0, query.length - 2);
      query += `)`;
      await this.execute(query);
      this.ensuredTables.push(entity.tableName);
    }
  }

  /**
   * Shortcut for safely executing a SQL query.
   * @param query - The query to execute.
   */
  private async execute(query: string) {
    console.log(query);
    return await (await this.getConnection()).execute(query);
  }

  /**
   * Checks if the given entity exists in the databse.
   * @param entity - The entity to check for.
   */
  public async exists(entity: Entity) {
    return (
      (
        await this.select(
          `SELECT * FROM ${entity.tableName} WHERE Id='${entity.id}'`,
        )
      ).length > 0
    );
  }

  /**
   * Gets a cleaned value.
   * @param value - The value to clean.
   */
  private getCleanValue(value: any) {
    if (typeof value === 'string') {
      return `'${value}'`;
    } else if (typeof value === 'number') {
      return `${value}`;
    } else {
      throw new Error(`Unhandled data type for ${value}!`);
    }
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
   * Inserts items into a table.
   * @param entity - The entity to insert.
   * @param autoUpdate - If the entity already exists, automatically update instead of inserting.
   */
  public async insert(entity: Entity, autoUpdate = true) {
    await this.ensureTable(entity);
    if (await this.exists(entity)) {
      if (autoUpdate) {
        this.update(entity);
      } else {
        throw new Error(
          `Entity already exists: ${entity.tableName}/${entity.id}`,
        );
      }
    } else {
      let query = `INSERT INTO ${entity.tableName} (Id,`;
      let values = `'${entity.id}',`;
      Object.keys(entity.data).forEach((name) => {
        query += `${name}, `;
        values += `${this.getCleanValue(entity.data[name])}, `;
      });
      query = query.substring(0, query.length - 2);
      values = values.substring(0, values.length - 2);
      query += `) VALUES (${values})`;
      await this.execute(query);
    }
  }

  /**
   * Performs a SELECT query.
   * @param query - The query to run.
   */
  private async select(query: string): Promise<any[]> {
    console.log(query);
    return await (await this.getConnection()).select(query);
  }

  /**
   * Selects and returns all entries from the given table.
   * @param from
   */
  public async selectAll<T extends Entity>(From: Constructor<T>): Promise<T[]> {
    try {
      const dummy = new From();
      return (await this.select(`SELECT * FROM ${dummy.tableName}`)).map(
        (result) => {
          const obj = new From(result);
          obj.id = result.Id;
          return obj;
        },
      );
    } catch (err) {
      return [];
    }
  }

  /**
   * Updates an entity.
   * @param entity - The entity to update.
   */
  public async update(entity: Entity) {
    await this.ensureTable(entity);
    let query = `UPDATE ${entity.tableName} SET `;
    Object.keys(entity.data).forEach((name) => {
      query += `${name} = ${this.getCleanValue(entity.data[name])}, `;
    });
    query = query.substring(0, query.length - 2);
    query += ` WHERE Id='${entity.id}'`;
    await this.execute(query);
  }
}
