export type PersonCategoryData = {
  id: string;
  name: string;
  color: string;
};

export type PersonCategoryRec = Record<PersonCategoryData['id'], PersonCategory>;

export class PersonCategory implements PersonCategoryData {
  public constructor(
    public readonly id: PersonCategoryData['id'],
    public _name: PersonCategoryData['name'],
    public _color: PersonCategoryData['color'],
  ) {}

  public get name() {
    return this._name;
  }

  public get color() {
    return this._color;
  }

  public static createCategories(categories: PersonCategoryData[]) {
    const mapped: PersonCategoryRec = {};
    for (const category of categories.map(
      ({ id, name, color }) => new PersonCategory(id, name, color),
    )) {
      mapped[category.id] = category;
    }
    return mapped;
  }
}
