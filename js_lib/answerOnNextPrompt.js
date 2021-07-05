function answerOnNextPrompt(answer) {
    var canUseLocalStorage = false;
    try { canUseLocalStorage = !!window.localStorage; } catch (ex) { /* probe failed */ }
    var canUseJSON = false;
    try { canUseJSON = !!JSON; } catch (ex) { /* probe failed */ }
    if (canUseLocalStorage && canUseJSON) {
        window.localStorage.setItem('__webdriverNextPrompt', JSON.stringify(answer));
    } else {
        window.__webdriverNextPrompt = answer;
    }
}