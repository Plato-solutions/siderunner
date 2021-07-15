window.onload = function () {
    document.getElementById('btn').addEventListener('dblclick', function (e) {
        document.getElementById('header').textContent = `${e.pageX} ${e.pageY} ${e.x} ${e.y} ${e.movementX} ${e.movementY}`;
    });

    document.getElementById('btn2').addEventListener('dblclick', function (_) {
        document.getElementById('header').textContent = `dblclick another button`;
    });
}