package keeper

import (
	"github.com/webb-tools/protocol-cosmos/x/anchor/types"
)

var _ types.QueryServer = Keeper{}
