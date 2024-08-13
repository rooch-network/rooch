import * as crypto from "crypto";
import { AbstractStartedContainer, GenericContainer, StartedTestContainer, Wait } from "testcontainers";

const BITCOIN_PORTS = [18443, 18444, 28333, 28332];

export class BitcoinContainer extends GenericContainer {
  private rpcBind = "0.0.0.0:18443";
  private rpcUser = "roochuser";
  private rpcPass = "roochpass";

  constructor(image = "lncm/bitcoind:v25.1") {
    super(image);
    this.withExposedPorts(...BITCOIN_PORTS).withStartupTimeout(120_000);
  }

  public withRpcBind(rpcBind: string): this {
    this.rpcBind = rpcBind;
    return this;
  }

  public withRpcUser(rpcUser: string): this {
    this.rpcUser = rpcUser;
    return this;
  }

  public withRpcPass(rpcPass: string): this {
    this.rpcPass = rpcPass;
    return this;
  }

  private generateRpcauth(): string {
    const salt = crypto.randomBytes(16).toString("hex");
    const hmac = crypto.createHmac("sha256", salt);
    hmac.update(this.rpcPass);
    const passwordHmac = hmac.digest("hex");

    return `${this.rpcUser}:${salt}$${passwordHmac}`;
  }

  public override async start(): Promise<StartedBitcoinContainer> {
    const rpcauth = this.generateRpcauth();

    this.withEnvironment({
      RPC_BIND: this.rpcBind,
      RPC_USER: this.rpcUser,
      RPC_PASS: this.rpcPass,
      RPC_AUTH: rpcauth,
    })
    .withWaitStrategy(Wait.forLogMessage("txindex thread start"))
    .withStartupTimeout(120000);

    this.withCommand([
      "-chain=regtest",
      "-txindex=1",
      "-fallbackfee=0.00001",
      "-zmqpubrawblock=tcp://0.0.0.0:28332",
      "-zmqpubrawtx=tcp://0.0.0.0:28333",
      "-rpcallowip=0.0.0.0/0",
      `-rpcbind=${this.rpcBind}`,
      `-rpcauth=${rpcauth}`,
    ])

    const container = await super.start();
    return new StartedBitcoinContainer(container, this.rpcBind, this.rpcUser, this.rpcPass);
  }
}

export class StartedBitcoinContainer extends AbstractStartedContainer {
  private readonly ports: { [key: number]: number };
  private preminedAddress: string | null = null;

  constructor(
    startedTestContainer: StartedTestContainer,
    private readonly rpcBind: string,
    private readonly rpcUser: string,
    private readonly rpcPass: string
  ) {
    super(startedTestContainer);
    this.ports = BITCOIN_PORTS.reduce((acc, port) => {
      acc[port] = startedTestContainer.getMappedPort(port);
      return acc;
    }, {} as { [key: number]: number });
  }

  public getPort(port: number): number {
    return this.ports[port];
  }

  public getRpcBind(): string {
    return this.rpcBind;
  }

  public getRpcUser(): string {
    return this.rpcUser;
  }

  public getRpcPass(): string {
    return this.rpcPass;
  }

  public getRpcUrl(): string {
    return `http://${this.getHost()}:${this.getPort(18443)}`;
  }

  public async executeRpcCommand(command: string, params: any[] = []): Promise<any> {
    return this.executeRpcCommandRaw([], command, params.map(param => JSON.stringify(param)))
  }

  public async executeRpcCommandRaw(opts: string[], command: string, params: string[] = []): Promise<any> {
    const cmd = [
      "bitcoin-cli",
      "-regtest",
      ...opts,
      command,
      ...params,
    ]

    const result = await this.startedTestContainer.exec(cmd);
    //console.log(`bitcoind run cmd: ${cmd.join(' ')}, result:${JSON.stringify(result)}`)

    if (result.exitCode !== 0) {
      throw new Error(`executeRpcCommand failed with exit code ${result.exitCode} for command: ${command}`);
    }
    return result.output;
  }

  public async prepareFaucet() {
    await this.executeRpcCommand("createwallet", ["faucet_wallet"])

    const getnewaddressOutput = await this.executeRpcCommandRaw([`-rpcwallet="faucet_wallet"`], "getnewaddress", []);
    this.preminedAddress = getnewaddressOutput.trim();

    if (this.preminedAddress) {
      await this.executeRpcCommandRaw([`-rpcwallet="faucet_wallet"`], "generatetoaddress", ["101", this.preminedAddress]);
    }
  }

  public async getFaucetBTC(address: string, amount: number = 0.001): Promise<string> {
    if (!this.preminedAddress) {
      throw new Error("Failed to generate pre-mined address");
    }

    const txid = await this.executeRpcCommandRaw([`-rpcwallet="faucet_wallet"`], "sendtoaddress", [address, "" + amount]);
    await this.executeRpcCommandRaw([`-rpcwallet="faucet_wallet"`], "generatetoaddress", ["1", this.preminedAddress]);

    return txid;
  }

  public async mineBlock(): Promise<void> {
    if (!this.preminedAddress) {
      throw new Error("Failed to generate pre-mined address");
    }

    await this.executeRpcCommandRaw([`-rpcwallet="faucet_wallet"`], "generatetoaddress", ["1", await this.preminedAddress]);
  }
}
