// SPDX-License-Identifier: MIT
pragma solidity 0.8.22;

contract HelloWorld {
    bytes data = abi.encode("Hello");

    function icphook(
        bytes memory _arguments
    ) public view returns (bytes memory) {
        return data;
    }
}
