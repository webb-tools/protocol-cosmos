package keeper_test

import (
	"context"
	"testing"

	sdk "github.com/cosmos/cosmos-sdk/types"
	keepertest "github.com/webb-tools/protocol-cosmos/testutil/keeper"
	"github.com/webb-tools/protocol-cosmos/x/anchor/keeper"
	"github.com/webb-tools/protocol-cosmos/x/anchor/types"
)

func setupMsgServer(t testing.TB) (types.MsgServer, context.Context) {
	k, ctx := keepertest.AnchorKeeper(t)
	return keeper.NewMsgServerImpl(*k), sdk.WrapSDKContext(ctx)
}
