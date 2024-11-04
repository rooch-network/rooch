export function openLink(url: string, tryInstantView: boolean = false) {
    // TODO: check desktop, web, mobile
    const data = JSON.stringify({
        url,
        try_instant_view: tryInstantView
    });
    window.TelegramWebviewProxy.postEvent('web_app_open_link', data)
}