export type PersonCategoryData = {
  id: string;
  name: string;
  color: string;
};

export class PersonCategory {
  public constructor(
    public readonly id: string,
    public _name: string,
    public _color: string,
  ) {}

  public get name() {
    return this._name;
  }

  public get color() {
    return this._color;
  }

  public static createCategories(categories: PersonCategoryData[]) {
    const mapped: Record<string, PersonCategory> = {};
    for (const category of categories.map(
      ({ id, name, color }) => new PersonCategory(id, name, color),
    )) {
      mapped[category.id] = category;
    }
    return mapped;
  }
}
