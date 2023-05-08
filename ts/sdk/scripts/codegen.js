const codegen = require("@cosmwasm/ts-codegen").default;
const path = require("path");
const fs = require("fs");

const pkgRoot = path.join(__dirname, "..");
const contractsDir = path.join(pkgRoot, "..", "..", "contracts");

const contracts = fs
  .readdirSync(contractsDir, { withFileTypes: true })
  .filter((c) => c.isDirectory())
  .map((c) => ({
    name: c.name,
    dir: path.join(contractsDir, c.name, "schema"),
  }));

const outPath = path.join(pkgRoot, "src", "contracts");
fs.rmSync(outPath, { recursive: true, force: true });

// patch missing description
contracts.forEach((contract) => {
  const mainSchemaFile = path.join(contract.dir, contract.name + ".json")
  const schema = JSON.parse(fs.readFileSync(mainSchemaFile));


  const updatedSchema = {
    ...schema,
    definitions: {
      Uint64: {
        description: "A thin wrapper around u64 that is using strings for JSON encoding/decoding, such that the full u64 range can be used for clients that convert JSON numbers to floats, like JavaScript and jq.\n\n# Examples\n\nUse `from` to create instances of this and `u64` to get the value out:\n\n``` # use cosmwasm_std::Uint64; let a = Uint64::from(42u64); assert_eq!(a.u64(), 42);\n\nlet b = Uint64::from(70u32); assert_eq!(b.u64(), 70); ```",
        type: "string"
      },
      Uint128: {
        description:
          "A thin wrapper around u128 that is using strings for JSON encoding/decoding, such that the full u128 range can be used for clients that convert JSON numbers to floats, like JavaScript and jq.\n\n# Examples\n\nUse `from` to create instances of this and `u128` to get the value out:\n\n``` # use cosmwasm_std::Uint128; let a = Uint128::from(123u128); assert_eq!(a.u128(), 123);\n\nlet b = Uint128::from(42u64); assert_eq!(b.u128(), 42);\n\nlet c = Uint128::from(70u32); assert_eq!(c.u128(), 70); ```",
        type: "string",
      },
      Timestamp: {
        description: "A point in time in nanosecond precision.\n\nThis type can represent times from 1970-01-01T00:00:00Z to 2554-07-21T23:34:33Z.\n\n## Examples\n\n``` # use cosmwasm_std::Timestamp; let ts = Timestamp::from_nanos(1_000_000_202); assert_eq!(ts.nanos(), 1_000_000_202); assert_eq!(ts.seconds(), 1); assert_eq!(ts.subsec_nanos(), 202);\n\nlet ts = ts.plus_seconds(2); assert_eq!(ts.nanos(), 3_000_000_202); assert_eq!(ts.seconds(), 3); assert_eq!(ts.subsec_nanos(), 202); ```",
        allOf: [
          {
            "$ref": "#/definitions/Uint64"
          }
        ]
      },
    }
  }


  fs.writeFileSync(mainSchemaFile, JSON.stringify(updatedSchema, null, 2));
})

codegen({
  contracts,
  outPath,
  options: {
    bundle: {
      bundleFile: "index.ts",
      scope: "contracts",
    },
  },
}).then(() => {
  console.log("✨ Typescript code is generated successfully!");
});
