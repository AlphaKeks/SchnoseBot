import SchnoseBot from "./classes/Schnose";
import "dotenv/config";

const schnose = new SchnoseBot({ intents: 34576 });
schnose.run(process.env.DJS_TOKEN);
