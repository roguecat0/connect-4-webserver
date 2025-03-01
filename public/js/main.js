import { initHome } from "./pages/home.js";
export function run() {
  let a = 10;
  let b = 32;
  let c = add(a, b);
  console.log({ a, b, c });
}
run();
function add(a, b) {
  return a + b;
}
$(function () {
  console.log("init function");
  initHome();
});
