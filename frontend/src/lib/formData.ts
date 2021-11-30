import {
  domainValidator,
  emailValidator,
  nonEmptyValidator,
  serverAddressValidator,
  Validated,
} from "./validators";

export enum AuthMethod {
  Password,
  Key,
}

export default class MainForm {
  // Cloudflare information
  public cfToken: Validated<string>;
  public cfEmail: Validated<string>;
  public cfZone: Validated<string>;
  public cfDns: Validated<string>;

  // SSH Server information
  public sshAddress: Validated<string>;
  public sshPort: Validated<number>;
  public sshUsername: Validated<string>;
  public sshAuthMethod: AuthMethod;
  public sshRsaKey: Validated<string>;
  public sshPassword: Validated<string>;

  constructor() {
    // Cloudflare information
    this.cfToken = new Validated("", nonEmptyValidator);
    this.cfEmail = new Validated("", emailValidator);
    this.cfZone = new Validated("", domainValidator);
    this.cfDns = new Validated("", domainValidator);

    // SSH Server information
    this.sshAddress = new Validated("", serverAddressValidator);
    this.sshPort = new Validated(22, nonEmptyValidator);
    this.sshUsername = new Validated("", nonEmptyValidator);
    this.sshRsaKey = new Validated("", nonEmptyValidator);
    this.sshPassword = new Validated("", nonEmptyValidator);
    this.sshAuthMethod = AuthMethod.Password;
  }

  public isValid(): boolean {
    const validity = [
      this.cfToken,
      this.cfEmail,
      this.cfZone,
      this.cfDns,
      this.sshAddress,
      this.sshPort,
      this.sshUsername,
    ].map((f) => f.isValid());
    if (this.sshAuthMethod == AuthMethod.Password) {
      return validity && this.sshPassword.isValid();
    } else {
      return validity && this.sshRsaKey.isValid();
    }
  }

  public toJson(): string {
    let d = {};
    for (const [k, v] of Object.entries(this)) {
      if (v instanceof Validated) d[k] = v.value;
      else d[k] = v;
    }
    return JSON.stringify(d);
  }
}
