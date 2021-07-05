// The credit for the function https://github.com/vmi/selenese-runner-java/blob/ca8511e6baa20939148edff2a139b4de2e1c11e7/src/main/resources/jp/vmi/selenium/selenese/javascript/JSLibrary.js#L28

function replaceAlertMethod(element) {
    if (!window.__isReplacedAlertMethod) {
        window.__isReplacedAlertMethod = true;
        var canUseLocalStorage = false;
        try { canUseLocalStorage = !!window.localStorage; } catch (ex) { /* probe failed */ }
        var canUseJSON = false;
        try { canUseJSON = !!JSON; } catch (ex) { /* probe failed */ }
        if (canUseLocalStorage && canUseJSON) {
            window.localStorage.setItem('__webdriverAlerts', JSON.stringify([]));
            window.alert = function (msg) {
                var alerts = JSON.parse(window.localStorage.getItem('__webdriverAlerts'));
                alerts.push(msg);
                window.localStorage.setItem('__webdriverAlerts', JSON.stringify(alerts));
            };
            window.localStorage.setItem('__webdriverConfirms', JSON.stringify([]));
            if (!('__webdriverNextConfirm' in window.localStorage))
                window.localStorage.setItem('__webdriverNextConfirm', JSON.stringify(true));
            window.confirm = function (msg) {
                var confirms = JSON.parse(window.localStorage.getItem('__webdriverConfirms'));
                confirms.push(msg);
                window.localStorage.setItem('__webdriverConfirms', JSON.stringify(confirms));
                var res = JSON.parse(window.localStorage.getItem('__webdriverNextConfirm'));
                window.localStorage.setItem('__webdriverNextConfirm', JSON.stringify(true));
                return res;
            };
            window.localStorage.setItem('__webdriverPrompts', JSON.stringify([]));
            if (!('__webdriverNextPrompt' in window.localStorage))
                window.localStorage.setItem('__webdriverNextPrompt', JSON.stringify(""));
            window.prompt = function (msg) {
                var prompts = JSON.parse(window.localStorage.getItem('__webdriverPrompts'));
                prompts.push(msg);
                window.localStorage.setItem('__webdriverPrompts', JSON.stringify(prompts));
                var res = JSON.parse(window.localStorage.getItem('__webdriverNextPrompt'));
                window.localStorage.setItem('__webdriverNextPrompt', JSON.stringify(""));
                return res;
            };
        } else {
            window.__webdriverAlerts = [];
            window.alert = function (msg) { window.__webdriverAlerts.push(msg); };
            window.__webdriverConfirms = [];
            if (typeof window.__webdriverNextConfirm === 'undefined')
                window.__webdriverNextConfirm = true;
            window.confirm = function (msg) {
                window.__webdriverConfirms.push(msg);
                var res = window.__webdriverNextConfirm;
                window.__webdriverNextConfirm = true;
                return res;
            };
            window.__webdriverPrompts = [];
            if (typeof window.__webdriverNextPrompt === 'undefined')
                window.__webdriverNextPrompt = true;
            window.prompt = function (msg, def) {
                window.__webdriverPrompts.push(msg || def);
                var res = window.__webdriverNextPrompt;
                window.__webdriverNextPrompt = true;
                return res;
            };
        }
    }
    var fw;
    if (element && (fw = element.ownerDocument.defaultView) && fw != window) {
        fw.alert = window.alert;
        fw.confirm = window.confirm;
        fw.prompt = window.prompt;
    }
}