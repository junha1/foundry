import { H256, PlatformAddress } from "../classes";
import { Transaction } from "../Transaction";
import { NetworkId } from "../types";

const RLP = require("rlp");

export class ChangeAssetScheme extends Transaction {
    private readonly _transaction: AssetSchemeChangeTransaction;
    private readonly approvals: string[];
    public constructor(input: {
        networkId: NetworkId;
        assetType: H256;
        metadata: string;
        approver: PlatformAddress | null;
        administrator: PlatformAddress | null;
        approvals: string[];
    }) {
        super(input.networkId);

        this._transaction = new AssetSchemeChangeTransaction(input);
        this.approvals = input.approvals;
    }

    public type(): string {
        return "changeAssetScheme";
    }

    protected actionToEncodeObject(): any[] {
        const encoded = this._transaction.toEncodeObject();
        encoded.push(this.approvals);
        return encoded;
    }

    protected actionToJSON(): any {
        const json = this._transaction.toJSON();
        json.approvals = this.approvals;
        return json;
    }
}

/**
 * Change asset scheme
 */
class AssetSchemeChangeTransaction {
    public readonly networkId: NetworkId;
    public readonly assetType: H256;
    public readonly metadata: string;
    public readonly approver: PlatformAddress | null;
    public readonly administrator: PlatformAddress | null;

    /**
     * @param params.networkId A network ID of the transaction.
     * @param params.assetType A asset type that this transaction changes.
     * @param params.metadata A changed metadata of the asset.
     * @param params.approver A changed approver of the asset.
     * @param params.administrator A changed administrator of the asset.
     */
    constructor(params: {
        networkId: NetworkId;
        assetType: H256;
        metadata: string;
        approver: PlatformAddress | null;
        administrator: PlatformAddress | null;
    }) {
        const {
            networkId,
            assetType,
            metadata,
            approver,
            administrator
        } = params;
        this.networkId = networkId;
        this.assetType = assetType;
        this.metadata = metadata;
        this.approver =
            approver === null ? null : PlatformAddress.ensure(approver);
        this.administrator =
            administrator === null
                ? null
                : PlatformAddress.ensure(administrator);
    }

    /**
     * Convert to an AssetSchemeChangeTransaction JSON object.
     * @returns An AssetSchemeChangeTransaction JSON object.
     */
    public toJSON(): any {
        return {
            networkId: this.networkId,
            assetType: this.assetType.toEncodeObject(),
            metadata: this.metadata,
            approver: this.approver == null ? null : this.approver.toString(),
            administrator:
                this.administrator == null
                    ? null
                    : this.administrator.toString()
        };
    }

    /**
     * Convert to an object for RLP encoding.
     */
    public toEncodeObject() {
        const {
            networkId,
            assetType,
            metadata,
            approver,
            administrator
        } = this;
        return [
            0x15,
            networkId,
            assetType,
            metadata,
            approver ? [approver.getAccountId().toEncodeObject()] : [],
            administrator ? [administrator.getAccountId().toEncodeObject()] : []
        ];
    }

    /**
     * Convert to RLP bytes.
     */
    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }
}
