// This app doesn't correctly refresh
import { promisify } from "node:util";

const sleep = promisify(setTimeout);

class App {
    private running = true;

    public constructor() {
        process.on("SIGINT", this.stop.bind(this));
    }

    public stop(): void {
        console.log("Caught interrupt signal");

        this.running = false;
    }

    public async printForever(): Promise<void> {
        while (this.running) {
            console.log(new Date().toISOString());

            await sleep(1000);
        }
    }
}

function main(): Promise<void> {
    const app = new App();

    return app.printForever();
}

await main();
