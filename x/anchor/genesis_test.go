package anchor_test

import (
	"testing"

	"github.com/stretchr/testify/require"
	keepertest "github.com/webb-tools/protocol-cosmos/testutil/keeper"
	"github.com/webb-tools/protocol-cosmos/x/anchor"
	"github.com/webb-tools/protocol-cosmos/x/anchor/types"
)

func TestGenesis(t *testing.T) {
	genesisState := types.GenesisState{
		PortId: types.PortID,
		// this line is used by starport scaffolding # genesis/test/state
	}

	k, ctx := keepertest.AnchorKeeper(t)
	anchor.InitGenesis(ctx, *k, genesisState)
	got := anchor.ExportGenesis(ctx, *k)
	require.NotNil(t, got)

	require.Equal(t, genesisState.PortId, got.PortId)
	// this line is used by starport scaffolding # genesis/test/assert
}
