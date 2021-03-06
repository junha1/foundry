// Copyright 2018-2019 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

export async function wait(duration: number) {
    await new Promise(resolve => setTimeout(() => resolve(), duration));
}

export class PromiseExpect {
    public waiting: { [index: string]: number };

    constructor() {
        this.waiting = {};
    }

    public async shouldFulfill<T>(key: string, p: Promise<T>): Promise<T> {
        if (!this.waiting[key]) {
            this.waiting[key] = 1;
        } else {
            this.waiting[key] += 1;
        }

        const result = await p;

        this.waiting[key] -= 1;
        if (this.waiting[key] === 0) {
            delete this.waiting[key];
        }
        return result;
    }

    public checkFulfilled() {
        const timeoutJobs = [];
        for (const key in this.waiting) {
            if (this.waiting.hasOwnProperty(key)) {
                timeoutJobs.push(key);
            }
        }
        this.waiting = {};

        if (timeoutJobs.length > 0) {
            throw new Error(
                `Timeout period is expired while waiting ${timeoutJobs.join(
                    ", "
                )}`
            );
        }
    }
}
