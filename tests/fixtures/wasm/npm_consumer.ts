import { run } from "host_portability";

declare const process: {
  env: Readonly<Record<string, string | undefined>>;
};

const exitCode: Promise<number> = run({
  args: ["host-portability", "first"],
  env: process.env,
  preopens: {},
});

void exitCode;
