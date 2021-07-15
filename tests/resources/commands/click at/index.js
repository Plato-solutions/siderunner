window.onload = function () {
    var button = document.getElementById('btn');
    button.onclick = function (e) {
        document.getElementById('header').textContent = `${e.pageX} ${e.pageY} ${e.x} ${e.y} ${e.movementX} ${e.movementY}`;
    }

    document.getElementById('btn2').onclick = function (e) {
        document.getElementById('header').textContent = `Click another button`;
    }
}