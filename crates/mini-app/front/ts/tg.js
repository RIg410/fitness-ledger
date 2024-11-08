export function tg_init() {
  console.log("tg init");
  Telegram.WebApp.ready();

  Telegram.WebApp.onEvent('themeChanged', function () {
    document.documentElement.className = Telegram.WebApp.colorScheme;
  });

  function setViewportData() {
    if (!Telegram.WebApp.isExpanded) {
      Telegram.WebApp.expand();
    }
  }

  Telegram.WebApp.setHeaderColor('secondary_bg_color');

  setViewportData();
  Telegram.WebApp.onEvent('viewportChanged', setViewportData);

  Telegram.WebApp.onEvent('themeChanged', function () {
    document.body.setAttribute('style', '--bg-color:' + Telegram.WebApp.backgroundColor);
  });
}

export function initData() {
  let data = Telegram.WebApp.initData;
  console.log("initData", data);
  return data;
}

export function alert(message) {
  Telegram.WebApp.showAlert(message);
}

export function showPopup(message, buttons, callback) {
  Telegram.WebApp.showPopup({
    title: 'SoulFamily',
    message: message,
    buttons: buttons
  }, callback);
}

export function close() {
  Telegram.WebApp.close();
}