export function run() {
  let a: number = 10;
  let b: number = 32;
  let c = add(a, b);
  console.log({ a, b, c });
}
run();

function add(a: number, b: number): number {
  return a + b;
}
$(function () {
  console.log("has jquery");
});
