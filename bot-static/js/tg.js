function app_init() {
  console.log('app_init');
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
