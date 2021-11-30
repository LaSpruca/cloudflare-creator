type Validator<T> = (source: T) => boolean;

export class Validated<T> {
  constructor(public value: T, private validator: Validator<T>) {}

  public isValid(): boolean {
    return this.value && this.validator(this.value);
  }
}

export const emailValidator: Validator<string> = (input: string) =>
  // eslint-disable-next-line no-control-regex
  /(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\.){3}(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])/gm
    .test(input);

export const domainValidator: Validator<string> = (input: string) =>
  /^(?!:\/\/)([a-zA-Z0-9-_]+\.)*[a-zA-Z0-9][a-zA-Z0-9-_]+\.[a-zA-Z]{2,11}?$/gm
    .test(input);

export const ipValidator: Validator<string> = (input: string) =>
  /^(?:(?:^|\.)(?:2(?:5[0-5]|[0-4]\d)|1?\d?\d)){4}$/gm.test(input);

export const serverAddressValidator: Validator<string> = (input: string) =>
  domainValidator(input) || ipValidator(input);

export const nonEmptyValidator = <T>(input: T): boolean =>
  input.toString() !== "";
