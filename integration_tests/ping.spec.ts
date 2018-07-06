import { SDK } from "../";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";

test("ping", async () => {
    const response = await new SDK({ server: SERVER_URL }).rpc.node.ping();
    expect(response).toBe("pong");
});
