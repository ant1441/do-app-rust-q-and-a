/*
 * Theme
 */
function setTheme() {
  document.documentElement.classList.toggle(
    "dark",
    localStorage.theme === "dark" ||
      (!("theme" in localStorage) &&
        window.matchMedia("(prefers-color-scheme: dark)").matches),
  );
}

function setLightTheme() {
  // Whenever the user explicitly chooses light mode
  localStorage.theme = "light";
  setTheme();
}
function setDarkTheme() {
  // Whenever the user explicitly chooses dark mode
  localStorage.theme = "dark";
  setTheme();
}
function setSystemTheme() {
  // Whenever the user explicitly chooses to respect the OS preference
  localStorage.removeItem("theme");
  setTheme();
}

window.addEventListener("DOMContentLoaded", setTheme);

/*
 * User Menu
 */

function toggleUserMenu() {
  const userMenu = document.getElementById("user-menu");
  userMenu.classList.toggle("hidden");
}
