window.onload = function () {
    const header = document.querySelector('#header');
    
    header.addEventListener('dblclick', function(e) {
        header.textContent = "Magic here";
    });
}