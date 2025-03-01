export function initHome() {
    console.log("init Home");
    $(document).on("click", "#toggle_show", function () {
        let showScores = $("#show_scores").val();
        let gameMoves = $("#game-moves").val();
        let res = get_name_value_pairs(["#game-moves", "#show_scores"]);
        $("#container").load("/toggle_show", res);
        console.log(res);
    });
    $(document).on("click", ".column", function () {
        let column = $(this);
        let columnNum = column.data("col");
        let path = column.data("path");
        let res = get_name_value_pairs([
            `ul[data-col='${columnNum}'] > [name='column']`,
            "[name='show_scores']",
        ]);
        console.log({ columnNum, path, res });
        $("#container").load(path, res);
    });
}
export function add2(a, b) {
    return a + b;
}
function get_name_value_pairs(cssSelectors) {
    let arr = cssSelectors.map((element) => {
        var _a;
        let inp = $(element);
        let name = (_a = inp.attr("name")) !== null && _a !== void 0 ? _a : "hello";
        let value = inp.val();
        return { name: name, value: value };
    });
    return arr.reduce((obj, item) => (Object.assign(Object.assign({}, obj), { [item.name]: item.value })), {});
}
