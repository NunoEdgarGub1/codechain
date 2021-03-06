// Copyright 2018 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

import { wait } from "../helper/promise";
import CodeChain from "../helper/spawn";

import "mocha";
import { expect } from "chai";

const BASE = 200;

describe("Memory pool size test", function() {
    let nodeA: CodeChain;
    const sizeLimit: number = 4;

    beforeEach(async function() {
        nodeA = new CodeChain({
            argv: ["--mem-pool-size", sizeLimit.toString()],
            base: BASE
        });
        await nodeA.start();
        await nodeA.sdk.rpc.devel.stopSealing();
    });

    it("To self", async function() {
        const sending = [];
        for (let i = 0; i < sizeLimit * 2; i++) {
            sending.push(
                nodeA.sendSignedParcel({ seq: i, awaitInvoice: false })
            );
        }
        await Promise.all(sending);
        const pendingParcels = await nodeA.sdk.rpc.chain.getPendingParcels();
        expect(pendingParcels.length).to.equal(sizeLimit * 2);
    }).timeout(10_000);

    describe("To others", async function() {
        let nodeB: CodeChain;

        beforeEach(async function() {
            nodeB = new CodeChain({
                argv: ["--mem-pool-size", sizeLimit.toString()],
                base: BASE
            });
            await nodeB.start();
            await nodeB.sdk.rpc.devel.stopSealing();

            await nodeA.connect(nodeB);
        });

        it("More than limit", async function() {
            for (let i = 0; i < sizeLimit * 2; i++) {
                await nodeA.sendSignedParcel({
                    seq: i,
                    awaitInvoice: false
                });
            }

            let counter = 0;
            while (
                (await nodeB.sdk.rpc.chain.getPendingParcels()).length <
                sizeLimit
            ) {
                await wait(500);
                counter += 1;
            }
            await wait(500 * (counter + 1));

            const pendingParcels = await nodeB.sdk.rpc.chain.getPendingParcels();
            expect(
                (await nodeB.sdk.rpc.chain.getPendingParcels()).length
            ).to.equal(sizeLimit);
        }).timeout(20_000);

        afterEach(async function() {
            await nodeB.clean();
        });
    });

    afterEach(async function() {
        await nodeA.clean();
    });
});

describe("Memory pool memory limit test", function() {
    let nodeA: CodeChain;
    const memoryLimit: number = 1;
    const mintSize: number = 5000;
    const sizeLimit: number = 5;

    beforeEach(async function() {
        nodeA = new CodeChain({
            chain: `${__dirname}/../scheme/mempool.json`,
            argv: ["--mem-pool-mem-limit", memoryLimit.toString()],
            base: BASE
        });
        await nodeA.start();
        await nodeA.sdk.rpc.devel.stopSealing();
    });

    it("To self", async function() {
        for (let i = 0; i < sizeLimit; i++) {
            await nodeA.mintAsset({ amount: 1, seq: i, awaitMint: false });
        }
        const pendingParcels = await nodeA.sdk.rpc.chain.getPendingParcels();
        expect(pendingParcels.length).to.equal(sizeLimit);
    }).timeout(50_000);

    describe("To others", async function() {
        let nodeB: CodeChain;

        beforeEach(async function() {
            nodeB = new CodeChain({
                chain: `${__dirname}/../scheme/mempool.json`,
                argv: ["--mem-pool-mem-limit", memoryLimit.toString()],
                base: BASE
            });
            await nodeB.start();
            await nodeB.sdk.rpc.devel.stopSealing();

            await nodeA.connect(nodeB);
        });

        it("More than limit", async function() {
            const [aBlockNumber, bBlockNumber] = await Promise.all([
                nodeA.sdk.rpc.chain.getBestBlockNumber(),
                nodeB.sdk.rpc.chain.getBestBlockNumber()
            ]);
            expect(aBlockNumber).to.equal(bBlockNumber);
            const metadata = "Very large parcel" + " ".repeat(1 * 1024 * 1024);
            const minting = [];
            for (let i = 0; i < sizeLimit; i++) {
                minting.push(
                    nodeA.mintAsset({
                        amount: mintSize,
                        seq: i,
                        metadata,
                        awaitMint: false
                    })
                );
            }
            await Promise.all(minting);
            await wait(3_000);

            const pendingParcels = await nodeB.sdk.rpc.chain.getPendingParcels();
            expect(pendingParcels.length).to.equal(0);
            expect(await nodeA.sdk.rpc.chain.getBestBlockNumber()).to.equal(
                aBlockNumber
            );
            expect(await nodeB.sdk.rpc.chain.getBestBlockNumber()).to.equal(
                bBlockNumber
            );
        }).timeout(50_000);

        afterEach(async function() {
            await nodeB.clean();
        });
    });

    afterEach(async function() {
        await nodeA.clean();
    });
});
