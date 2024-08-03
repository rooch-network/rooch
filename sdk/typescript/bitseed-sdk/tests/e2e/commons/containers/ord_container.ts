import { AbstractStartedContainer, GenericContainer, StartedTestContainer, Wait } from "testcontainers";

const ORD_PORT = 80;

export class OrdContainer extends GenericContainer {
  private bitcoinRpcUrl = "";
  private bitcoinRpcUser = "";
  private bitcoinRpcPass = "";

  constructor(image = "bitseed/ord:0.18.0-burn") {
    super(image);
    this.withExposedPorts(ORD_PORT)
      .withStartupTimeout(120_000)
      .withWaitStrategy(Wait.forLogMessage("Listening on"));
  }

  public withBitcoinRpcUrl(url: string): this {
    this.bitcoinRpcUrl = url;
    return this;
  }

  public withBitcoinRpcUser(user: string): this {
    this.bitcoinRpcUser = user;
    return this;
  }

  public withBitcoinRpcPass(pass: string): this {
    this.bitcoinRpcPass = pass;
    return this;
  }

  public override async start(): Promise<StartedOrdContainer> {
    this.withCommand([
      "--regtest",
      `--bitcoin-rpc-url=${this.bitcoinRpcUrl}`,
      `--bitcoin-rpc-username=${this.bitcoinRpcUser}`,
      `--bitcoin-rpc-password=${this.bitcoinRpcPass}`,
      "server"
    ]);

    const startedContainer = await super.start();
    
    // Execute the command after start
    await startedContainer.exec(["/bin/rm", "-rf", "/data/.bitcoin/regtest/wallets/ord"]);

    return new StartedOrdContainer(
      startedContainer,
      this.bitcoinRpcUrl,
      this.bitcoinRpcUser,
      this.bitcoinRpcPass
    );
  }
}

export class StartedOrdContainer extends AbstractStartedContainer {
  private readonly port: number;

  constructor(
    startedTestContainer: StartedTestContainer,
    private readonly bitcoinRpcUrl: string,
    private readonly bitcoinRpcUser: string,
    private readonly bitcoinRpcPass: string
  ) {
    super(startedTestContainer);
    this.port = startedTestContainer.getMappedPort(ORD_PORT);
  }

  public getPort(): number {
    return this.port;
  }

  public getBitcoinRpcUrl(): string {
    return this.bitcoinRpcUrl;
  }

  public getBitcoinRpcUser(): string {
    return this.bitcoinRpcUser;
  }

  public getBitcoinRpcPass(): string {
    return this.bitcoinRpcPass;
  }

  public getConnectionUri(): string {
    return `http://${this.getHost()}:${this.getPort()}`;
  }
}