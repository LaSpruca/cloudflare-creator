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

/** A class to wrap all of the information in the main form */
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

  /** Checks the validity of all the fields in the form */
  public isValid(): boolean {
    // Check the validity of all of the main fields
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
      // If using password authentication, check that the password is valid
      return validity && this.sshPassword.isValid();
    } else {
      // If using RSA Key authentication, check that the key is valid
      return validity && this.sshRsaKey.isValid();
    }
  }

  /** Unwraps all Validated objects and returns the value */
  public toJson(): string {
    // Create a temporary variable to store all the fields of the object
    const d = {};
    for (const [k, v] of Object.entries(this)) {
      // If item is a Validated, add the value to the object
      if (v instanceof Validated) d[k] = v.value;
      // If the item is not a Validated, directly add it to the object
      else d[k] = v;
    }

    if (this.sshAuthMethod == AuthMethod.Key) {
      d["sshPassword"] = null;
    } else {
      d["sshRsaKey"] = null;
    }

    // Convert it to a json object
    return JSON.stringify(d);
  }
}
