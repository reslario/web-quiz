function format(seconds) {
    let date = new Date(0);
    date.setSeconds(seconds);
    return date
        .toISOString()
        .substr(11, 8);
}

function startTimer(elapsed) {
    let timer = document.getElementById("timer");
    updateTimer(elapsed);
    setInterval(
        () => updateTimer(elapsed++),
        1000
    )
}

function updateTimer(elapsed) {
    timer.innerText = format(elapsed);
}