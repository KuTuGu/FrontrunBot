// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "contract/Arbitrage.sol";

interface IUniswapV2Router {
    function swapExactTokensForTokensSupportingFeeOnTransferTokens(
        uint256 amountIn,
        uint256 amountOutMin,
        address[] calldata path,
        address to,
        uint256 deadline
    ) external;
}

interface IUniswapV2Pair {
    function token0() external view returns (address);
    function token1() external view returns (address);
    function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
}

contract NotFlationTokenTest is Test {
    struct Token {
        address addr;
        address lp;
    }

    struct BalanceInfo {
        uint256 from;
        uint256 to;
        uint256 lp;
        uint256 base_reverse;
        uint256 pair_reverse;
    }

    mapping(address => bool) public notFlationTokenMap;
    address router = 0x10ED43C718714eb63d5aA57B78B54704E256024E;

    function testETHNotFlationToken1() public {
        vm.createSelectFork("https://rpc.ankr.com/eth", 16428369);
        // 0xa806617cdd8ed760ed25cec61abf642f4889749c3cede45c46f27d60f0941bd1
        address QTN = 0xC9fa8F4CFd11559b50c5C7F6672B9eEa2757e1bd;
        // 0xd099a41830b964e93415e9a8607cd92567e40d3eeb491d52f3b66eee6b0357eb
        address UPStkn = 0xFFeE5EcDE135a7b10A0Ac0E6e617798e6aD3D0D6;
        address DBS = 0x4f7AFf8f0c78B51c0E30F02f27a67B5A6A11552b;
        address PRX = 0xE8847D2fA66D0D1f4A77221caE1e47d8d59CF7d7;
        address WETH = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;
        Token[5] memory TOKEN_LIST = [
            Token(QTN, 0xA8208dA95869060cfD40a23eb11F2158639c829B),
            Token(UPStkn, 0xa3f47DCFC09d9aaDb7Ac6ECAB257cf7283BFEe26),
            Token(DBS, 0xd4192eE224e111Dc39098f93e5D62f9469D8842c),
            Token(PRX, 0xD3a3b04E222229Ec1DD1215363b7ac5E0102eB8e),
            Token(WETH, 0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc)
        ];

        for (uint256 i = 0; i < TOKEN_LIST.length; i++) {
            // ignore error
            address(this).call(abi.encodeWithSignature("check((address,address))", TOKEN_LIST[i]));
        }

        // Unable to detect irregular contracts
        assert(notFlationTokenMap[QTN] == false);

        // transfer self increment
        assert(notFlationTokenMap[UPStkn] == true);

        // weth is safety
        assert(notFlationTokenMap[WETH] == false);
    }

    function testETHNotFlationToken2() public {
        vm.createSelectFork("https://rpc.ankr.com/eth");
        address HIMEI = 0x81b6E6EE0Bd303A1f1Ef7D63f9A071F7eF2abe09;

        Token[1] memory TOKEN_LIST = [Token(HIMEI, 0x4a449fFD26332170b19450FB573864407385B2d4)];

        for (uint256 i = 0; i < TOKEN_LIST.length; i++) {
            // ignore error
            address(this).call(abi.encodeWithSignature("check((address,address))", TOKEN_LIST[i]));
        }

        // slight lp reverse change are also detected, even is safety
        assert(notFlationTokenMap[HIMEI] == true);
    }

    function testBSCNotFlationToken() public {
        vm.createSelectFork("https://rpc.ankr.com/bsc", 24912666);
        // 0x5058c820fa0bb0daff2bd1b30151cf84c618dffe123546223b7089c8c2e18331
        address THOREUM = 0x79Fe086a4C03C5E38FF8074DEA9Ee0a18dC1AF4F;
        address WBNB = 0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c;
        Token[2] memory TOKEN_LIST = [
            Token(THOREUM, 0xd822E1737b1180F72368B2a9EB2de22805B67E34),
            Token(WBNB, 0x1CEa83EC5E48D9157fCAe27a19807BeF79195Ce1)
        ];

        for (uint16 i = 0; i < TOKEN_LIST.length; i++) {
            // ignore error
            address(this).call(abi.encodeWithSignature("check((address,address))", TOKEN_LIST[i]));
        }

        // not a simple transfer self increment, the specific arbitrage logic is not clear
        assert(notFlationTokenMap[THOREUM] == false);

        // wbnb is safety
        assert(notFlationTokenMap[WBNB] == false);
    }

    function testBSCSafemoonToken1() public {
        vm.createSelectFork("https://rpc.ankr.com/bsc", 24912666);
        address DBALL = 0x4b0b131C91CD5C1617A8853FeA7bCda5640495B8;
        address SHEEP = 0x0025B42bfc22CbbA6c02d23d4Ec2aBFcf6E014d4;
        address NWORDPASS = 0x294ed90b2915DEb5753328916d4e229a9930fFC2;
        address GFFT = 0x87d4773CFfc30D064A8d139429b04284Ee26c789;
        address BNG = 0x6010e1a66934C4D053E8866Acac720c4a093d956;
        address JUST = 0x54e3C6c6386aD3Ccc071DF34464F97DA1a4230d9;
        address LXR = 0x3f3F079d6F464F31D04463ca6abBb539b9a844f6;
        address NyQuil = 0xc86875d594f2CDF3ad15cac093A005981624C65E;
        address CAMP = 0xCb48D94AD31ea5238446024B05f95c4512b6eA32;
        address DOLLR = 0x22a55375c545550270f08497c4E7D9E033607D9C;
        Token[10] memory TOKEN_LIST = [
            Token(DBALL, 0x4961c488d8Ec130e8CBD3c73966a6530afeEB53d),
            Token(SHEEP, 0x912DCfBf1105504fB4FF8ce351BEb4d929cE9c24),
            Token(NWORDPASS, 0x828d77206AE4BAd53606F6470844Ee93400fE4F2),
            Token(GFFT, 0xC71DB029b3f80523a7aCF88962fAE69BFa9a3c09),
            Token(BNG, 0xb533BB48e274BA3972B95cF150762C5F13bd2567),
            Token(JUST, 0x5DF0c8f0849b3F98368fB609FC9CcD397bC60889),
            Token(LXR, 0x627CD8A897667A2A46601B0c93f2cB3bBEE3484d),
            Token(NyQuil, 0x3D51b4bcaDAD9485724e6e188638b50b52708d60),
            Token(CAMP, 0xa8BB109B94DEF015B34b79CE9Fcf8DFf42D7203D),
            Token(DOLLR, 0xD79dd9612151df1687122300F0Dcd7D4cEd2DC4e)
        ];

        for (uint16 i = 0; i < TOKEN_LIST.length; i++) {
            // ignore error
            address(this).call(abi.encodeWithSignature("check((address,address))", TOKEN_LIST[i]));
        }
    }

    function testBSCSafemoonToken2() public {
        vm.createSelectFork("https://rpc.ankr.com/bsc", 24912666);
        address BOO = 0x47cDD168234BDEFf21648E1463caA38Bb95403DE;
        address SMARTWORTH = 0xD50787A5f21bcC10C1a738E7DE33001786c5fc24;
        address PUPPY = 0xB8D14376A2dA9E500Aa8Fa2537351c0440e06C61;
        address Lfloki = 0x15681938b019D463e06715142e48538e496BCA09;
        Token[4] memory TOKEN_LIST = [
            Token(BOO, 0x13d7922F4B93EDD29F740bf4139Bd0821c07c4c9),
            Token(SMARTWORTH, 0xB9cc2Df65d04a5e07b2b6c9A8D3048000f9208a9),
            Token(PUPPY, 0x5C89c9Ba27899a29ba8e51f6aE1231868A35E46E),
            Token(Lfloki, 0xFa91676B3C43CA55D8266eC5a1023C292013135d)
        ];

        for (uint16 i = 0; i < TOKEN_LIST.length; i++) {
            _check(TOKEN_LIST[i]);
        }
    }

    function _check(Token memory _token) internal {
        address token = _token.addr;
        address lp = _token.lp;
        address token0 = IUniswapV2Pair(lp).token0();
        address token1 = IUniswapV2Pair(lp).token1();
        bool swap = token == token1;
        address pairToken = swap ? token0 : token1;

        // set balance
        uint256 amount = 1 ether;
        deal(pairToken, address(this), amount);
        IERC20(pairToken).approve(router, amount);
        address[] memory path = new address[](2);
        path[0] = pairToken;
        path[1] = token;
        IUniswapV2Router(router).swapExactTokensForTokensSupportingFeeOnTransferTokens(
            amount, 0, path, address(this), block.timestamp + 1 minutes
        );

        amount = IERC20(token).balanceOf(address(this)) / 2;
        // safemoon exploit, change _rTotal
        _burn(_token, address(this), address(0), amount);
        // transfer from this to lp pool
        _transfer(_token, address(this), address(lp), amount);
        // transfer from lp pool to this
        _transfer(_token, address(lp), address(this), amount);
        // UPStkn exploit, transfer self with zero amount
        _transfer(_token, address(this), address(this), 0);
        // THOREUM exploit, transfer self with non-zero amount
        _transfer(_token, address(this), address(this), amount);
    }

    function _transfer(Token memory _token, address from, address to, uint256 amount) internal {
        BalanceInfo memory balance_before = _getBalanceInfo(_token, from, to);

        vm.prank(from);
        address(_token.addr).call(abi.encodeWithSignature("transfer(address,uint256)", to, amount));

        BalanceInfo memory balance_after = _getBalanceInfo(_token, from, to);

        if (
            (from != to && balance_after.from != balance_before.from - amount)
                || (from == to && balance_after.from > balance_before.from)
                || (to != _token.lp && balance_after.to > (from != to ? balance_before.to + amount : balance_before.to))
                || (
                    balance_after.base_reverse * balance_before.pair_reverse
                        != balance_after.pair_reverse * balance_before.base_reverse
                )
        ) {
            notFlationTokenMap[_token.addr] = true;
        }
    }

    function _burn(Token memory _token, address from, address to, uint256 amount) internal {
        BalanceInfo memory balance_before = _getBalanceInfo(_token, from, to);

        vm.startPrank(from);
        address(_token.addr).call(abi.encodeWithSignature("burn(uint256)", amount / 2));
        address(_token.addr).call(abi.encodeWithSignature("deliver(uint256)", amount / 2));
        vm.stopPrank();

        BalanceInfo memory balance_after = _getBalanceInfo(_token, from, to);
        console.log(balance_before.lp, balance_after.lp);
        console.log(balance_before.from, balance_after.from);
        console.log(balance_before.base_reverse, balance_after.base_reverse);
        console.log(balance_before.pair_reverse, balance_after.pair_reverse);
    }

    function _getBalanceInfo(Token memory _token, address from, address to)
        internal
        view
        returns (BalanceInfo memory balanceInfo)
    {
        address token = _token.addr;
        address lp = _token.lp;
        bool swap = token == IUniswapV2Pair(lp).token1();
        (uint256 reverse0, uint256 reverse1,) = IUniswapV2Pair(lp).getReserves();

        balanceInfo = BalanceInfo(
            IERC20(token).balanceOf(from),
            IERC20(token).balanceOf(to),
            IERC20(token).balanceOf(lp),
            swap ? reverse1 : reverse0,
            swap ? reverse0 : reverse1
        );
    }
}
