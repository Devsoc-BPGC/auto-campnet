const { emit,listen } = window.__TAURI__.event;

document.getElementById('save').addEventListener('click',_ => {
    document.getElementById("saved_message").className = "creds-saved";
    emit('save', {
        username: encodeURIComponent(document.getElementById('loginuserid').value),
        password: encodeURIComponent(document.getElementById('loginpassword').value)
    });
    setTimeout(() =>
        document.getElementById("saved_message").className = "creds-saved hide",
        3000);
});

listen('credentials', creds => {
    document.getElementById('loginuserid').value = decodeURIComponent(creds.payload.Ok.username);
    document.getElementById('loginpassword').value = decodeURIComponent(creds.payload.Ok.password);
});
