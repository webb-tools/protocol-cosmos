use crate::*;


#[derive(Clone, Encode, Decode, TypeInfo)]
pub struct AnchorMetadata<AccountId, Balance, AssetId> {
	/// Creator account
	pub creator: AccountId,
	/// Balance size of deposit
	pub deposit_size: Balance,
	/// Option of specifying a fungible asset. When None, the asset is the
	/// native currency.
    // TODO find out what kind assetId to use from cosmos , see cosmos tx example below
	pub asset: AssetId,
}

/*
   Cosmos tx example
{ 
  "body": {
    "messages": [
      {
        "@type": "/cosmos.bank.v1beta1.MsgSend",
        "from_address": "cosmos1snd5m4h0wt5ur55d47vpxla389r2xkf8dl6g9w",
        "to_address": "cosmos1t2e0nyjhwn3revunvf2uperhftvhzu4euuzva9",
        "amount": [
          {
            "denom": "basecoin",
            "amount": "100"
          },
          {
            "denom": "othercoin",
            "amount": "1000"
          }
        ]
      }
    ],
    "memo": "",
    "timeout_height": "0",
    "extension_options": [],
    "non_critical_extension_options": []
  },
  "auth_info": {
    "signer_infos": [],
    "fee": {
      "amount": [],
      "gas_limit": "200000",
      "payer": "",
      "granter": ""
    }
  },
  "signatures": []
}
*/