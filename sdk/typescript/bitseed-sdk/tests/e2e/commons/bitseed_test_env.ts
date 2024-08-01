import {
  TestContainer,
  StartedTestContainer,
  GenericContainer
} from "testcontainers";

export class BitseedTestEnv {
  container: TestContainer
  startedContainer: StartedTestContainer

  constructor() {
    this.container = new GenericContainer("alpine");
  }

  async start() {
    console.log('container start');
    this.startedContainer = await this.container.start();
  }

  async stop() {
    console.log('container stop');
    await this.startedContainer.stop();
  }
}
