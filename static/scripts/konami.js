(function() {
    var count = 0;
    var messages = [
        "Well done, you won the game",
        "No seriously you won, you can stop now",
        "Just stop doing that, it's bad for health",
        "Seriously you'll have to see your GP if you keep this up"
    ];
    new Konami(function() {
        var post = document.body.getElementsByClassName('post')[0];
        post.textContent = messages[count] || messages[0];
        count += 1;
    });
})()
