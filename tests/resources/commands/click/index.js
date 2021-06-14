window.onload = function () {
    var button = document.getElementById('btn');
    button.onclick = function () {
        document.getElementById('header').textContent = "Header:Clicked";
    }
}