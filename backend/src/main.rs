use std::time::Instant;

static START: Instant = Instant::now();
static PROGRAM: &str = r#"import axiod from "https://deno.land/x/axiod/mod.ts";

const api_base = "https://api.cloudflare.com/client/v4";

const headers = {
  "X-Auth-Email": "nathanhare32@gmail.com",
  "Authorization": "Bearer " + token,
  "Content-Type": "application/json",
};

type Zone = {
  id: string;
  name: string;
};

type Dns = {
  id: string;
  name: string;
  type: string;
  content: string;
};

try {
  const { data: zones } = await axiod.get(api_base + "/zones", { headers });

  const zoneId = zones.result.filter((f: Zone) =>
    f.name == CF_ZONE
  ).map((f: Zone) => f.id)[0];

  const { data: dns } = await axiod.get(
    api_base + "/zones/" + zoneId + "/dns_records/",
    { headers },
  );

  const dnsId = dns.result.filter((f: Dns) =>
    f.name == CF_RECORD && f.type == "A"
  ).map((f: Dns) => {
    return { id: f.id, ip: f.content };
  })[0];

  const ip: any = {};
  (await axiod.get("https://cloudflare.com/cdn-cgi/trace"))
    .data
    .split("\n").forEach((f: string) => {
      const [key, value] = f.split("=");
      ip[key] = value;
    });

  console.log("Couldflare IP is", dnsId.ip);
  console.log("Current IP is " + ip.ip);

  if (dnsId.ip != ip.ip) {
    console.log("Updating Cloudflare IP");
    await axiod.put(
      api_base + "/zones/" + zoneId + "/dns_records/" + dnsId.id,
      {
        type: "A",
        name: "devtron.laspruca.nz",
        content: ip.ip,
        ttl: 1,
      },
      {
        headers,
      },
    );
    console.log("Updated IP Address");
  } else {
    console.log("IP is the same, skipping update");
  }
} catch (ex) {
  console.log("Caught Exception");
  console.log(ex.response.data);
}
"#;

fn main() {
    let source = create_function("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX".into(), "laspruca.nz".into(), "devtron.laspruca.nz".into());
    println!("{}", source);
}

fn create_function(cf_token: String, cf_zone: String, cf_domain: String) -> String {
    let source = format!(
        "const CF_TOKEN = \"{}\";\nconst CF_ZONE = \"{}\";\nconst CF_DOMAIN = \"{}\";\n{}",
        cf_token, cf_zone, cf_domain, PROGRAM
    );

    let filename = format!("i-{}", std::time::Instant::now());

    std::fs::write("o-{")
}
