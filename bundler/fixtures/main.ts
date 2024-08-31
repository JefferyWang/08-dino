import { execute } from "./lib.ts";

async function main() {
    console.log('Executing main.ts');
    console.log(await execute('world'));
}

export default main;
