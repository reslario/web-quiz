async function useJoker() {
    let resp = await fetch("/play/use_joker", { credentials: "include" });
    if (resp.ok) {
        applyJoker(await resp.json())
    }
}

function applyJoker(joker) {
    document.getElementById("answers")
        .childNodes
        .forEach(answer => {
            if (joker.incorrect.includes(answer.value)) {
                answer.disabled = true;
            }
        });
    removeJokerButton();
}

function removeJokerButton() {
    let button = document.getElementById("joker");
    button
        .parentNode
        .removeChild(button);
}