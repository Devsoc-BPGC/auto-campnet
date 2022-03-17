const { emit } = window.__TAURI__.event;

document.getElementById('save').addEventListener('click',_ => {
    emit('save', {
        username: encodeURIComponent(document.getElementById('loginuserid').value),
        password: encodeURIComponent(document.getElementById('loginpassword').value)
    });
});
