import { hashToHex } from "@lib/utils/hash";

self.addEventListener(
  "message",
  async function (e: { data: { challenge: string } }) {
    const criteria = e.data?.challenge;
    if (!criteria) return;
    const difficulty = Number.parseInt(criteria.split("#")[0], 10);
    const challenge = criteria.split("#")[1];
    let nonce = 0;
    // console.log(`difficulty: ${difficulty}, challenge: ${challenge}, nonce: ${nonce}, criteria: ${criteria}`)
    let result = "";
    while (!result.startsWith(new Array(difficulty + 1).join("0"))) {
      nonce++;
      result = await hashToHex(new TextEncoder().encode(challenge + nonce.toString(16)));
      // console.log(`nonce: ${nonce}, hash: ${result}`)
    }
    this.postMessage(challenge + nonce.toString(16));
  },
  false
);
