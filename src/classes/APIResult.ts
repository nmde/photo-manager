export class APIResult<T, U = T> {
  private ok_fn!: (value: U) => void;
  private err_fn!: (message: string) => void;

  public constructor(
    private fn: () => Promise<T>,
    private transformer: (data: T) => U = x => x as unknown as U,
  ) {}

  public ok(fn: (value: U) => void) {
    this.ok_fn = fn;
    return this;
  }

  public err(fn: (message: string) => void) {
    this.err_fn = fn;
    return this;
  }

  public async send() {
    try {
      this.ok_fn(this.transformer(await this.fn()));
    } catch (error) {
      this.err_fn((error as Error).message);
    }
  }
}
