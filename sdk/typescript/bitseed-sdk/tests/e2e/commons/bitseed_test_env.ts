import { BitcoinContainer, StartedBitcoinContainer } from "./containers/bitcoin_container";
import { RoochContainer, StartedRoochContainer } from "./containers/rooch_container";
import { Network, StartedNetwork } from "testcontainers";

export class BitseedTestEnv {
  bitcoinContainer: BitcoinContainer;
  startedBitcoinContainer: StartedBitcoinContainer | null = null;
  roochContainer: RoochContainer;
  startedRoochContainer: StartedRoochContainer | null = null;

  private network: Network;
  private startedNetwork: StartedNetwork;
  private readonly bitcoinNetworkAlias = "bitcoind";
  private miningIntervalId: NodeJS.Timeout | null = null;

  constructor() {
    this.network = new Network();
    this.bitcoinContainer = new BitcoinContainer();
    this.roochContainer = new RoochContainer();
  }

  async start() {
    console.log('Starting containers');

    this.startedNetwork = await this.network.start();

    // Start Bitcoin container
    this.startedBitcoinContainer = await this.bitcoinContainer
      .withNetwork(this.startedNetwork)
      .withNetworkAliases(this.bitcoinNetworkAlias)
      .start();

    console.log('Bitcoin container started');

    // Preprea Faucet
    await this.startedBitcoinContainer.prepareFaucet();

    this.roochContainer.withHostConfigPath('/tmp/.rooch')
    await this.roochContainer.initializeRooch();
    console.log('Rooch container init');

    // Start Rooch container with Bitcoin RPC configuration
    this.roochContainer
      .withNetwork(this.startedNetwork)
      .withNetworkName("local")
      .withDataDir("TMP")
      .withPort(6767)
      .withBtcRpcUrl("http://bitcoind:18443")
      .withBtcRpcUsername(this.startedBitcoinContainer.getRpcUser())
      .withBtcRpcPassword(this.startedBitcoinContainer.getRpcPass())
      .withBtcSyncBlockInterval(1); // Set sync interval to 1 second

    this.startedRoochContainer = await this.roochContainer.start();
    console.log('Rooch container started');

    // Start mining interval after Bitcoin container is started
    this.miningIntervalId = setInterval(async () => {
      if (this.startedBitcoinContainer) {
        await this.startedBitcoinContainer.mineBlock();
      }
    }, 1000); // Mine every 1 second
  }

  async stop() {
    console.log('Stopping containers');

    // Clear mining interval before stopping containers
    if (this.miningIntervalId) {
      clearInterval(this.miningIntervalId);
      this.miningIntervalId = null;
    }

    // Stop Rooch container
    if (this.startedRoochContainer) {
      await this.startedRoochContainer.stop();
      console.log('Rooch container stopped');
    }

    // Stop Bitcoin container
    if (this.startedBitcoinContainer) {
      await this.startedBitcoinContainer.stop();
      console.log('Bitcoin container stopped');
    }

    // Stop the network
    await this.startedNetwork.stop();
    console.log('Network stopped');

    // Reset container references
    this.startedRoochContainer = null;
    this.startedBitcoinContainer = null;
  }

  /**
   * Get the Rooch server listening address
   * @returns The URI of the Rooch server, or null if the server is not running
   */
  getRoochServerAddress(): string | null {
    if (this.startedRoochContainer) {
      return this.startedRoochContainer.getConnectionAddress();
    }

    return null;
  }

  async getFaucetBTC(address: string, amount: number = 0.001): Promise<string> {
    if (!this.startedBitcoinContainer) {
      throw new Error("bitcoin container not start")
    }

    return await this.startedBitcoinContainer.getFaucetBTC(address, amount);
  }
}