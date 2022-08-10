import mongoose from "mongoose";

const userSchema = new mongoose.Schema({
	name: String,
	discordID: String,
	steamID: String,
	mode: String
});

export default mongoose.model("user", userSchema);
