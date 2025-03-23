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

window.addEventListener("DOMContentLoaded", function () {
  document
    .getElementById("set-light-theme")
    .addEventListener("click", setLightTheme);
  document
    .getElementById("set-dark-theme")
    .addEventListener("click", setDarkTheme);
  document
    .getElementById("set-system-theme")
    .addEventListener("click", setSystemTheme);
});

/*
 * Forms
 */

window.addEventListener("DOMContentLoaded", function () {
  const submitTopicForm = document.getElementById("submit-topic");
  if (submitTopicForm === null) {
    return;
  }

  submitTopicForm.addEventListener("submit", async function (e) {
    e.preventDefault();

    const submitter = e.submitter;

    const formData = new FormData(submitTopicForm, submitter);
    const topic = formData.get("topic");

    const response = await fetch("/api/set_topic", {
      method: "post",
      headers: {
        Accept: "application/json",
        "Content-Type": "application/json",
      },

      body: JSON.stringify({
        topic: topic,
      }),
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new Error(
        `Error ${response.status}: ${errorData.reason || "Unknown error"}`,
      );
    }

    console.log("Topic set successfully:", topic);
    location.reload();
  });
});
