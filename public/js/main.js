import { initHome, add2 } from "./pages/home.js";
export function run() {
    let a = 10;
    let b = 32;
    let c = add(a, b);
    let c2 = add2(a, b);
    console.log({ a, b, c, c2 });
}
run();
function add(a, b) {
    return a + b;
}
$(function () {
    console.log("init function");
    initHome();
});
