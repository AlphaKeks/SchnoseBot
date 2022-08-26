import SchnoseBot from "./classes/Schnose";
import "dotenv/config";

const schnose = new SchnoseBot({ intents: 34619 });
schnose.run(process.env.DJS_TOKEN);
