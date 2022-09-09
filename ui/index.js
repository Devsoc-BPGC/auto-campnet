const { emit, listen } = window.__TAURI__.event;

document.getElementById("save").addEventListener("click", (_) => {
    document.getElementById("saved_message").className = "creds-saved";
    emit("save", {
        username: encodeURIComponent(
            document.getElementById("loginuserid").value
        ),
        password: encodeURIComponent(
            document.getElementById("loginpassword").value
        ),
    });
    setTimeout(
        () =>
            (document.getElementById("saved_message").className =
                "creds-saved hide"),
        3000
    );
});

listen("credentials", (creds) => {
    document.getElementById("loginuserid").value = decodeURIComponent(
        creds.payload.Ok.username
    );
    document.getElementById("loginpassword").value = decodeURIComponent(
        creds.payload.Ok.password
    );
});

document.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "hidden") emit("minimise");
});

//set using backend
const campus = "goa";

const campusImages = {
    goa:"https://180dc.org/wp-content/uploads/2016/04/bits-goa-campus.jpg",
    pilani:"https://www.bits-pilani.ac.in/Uploads/Pilani/pilanimanagementadmin/Gallery/2019-3-3--0-59-3-979_22158613_310339669446805_463247969886404608_n.jpg",
    hyderabad:"https://www.bits-pilani.ac.in/Uploads/Hyderabad/adminforhyderabad/Gallery/2015-6-25--15-1-11-412_audi1.JPG"
}; 

const campusLogos = {
    goa:"https://www.bits-pilani.ac.in/Uploads/Campus/BITS_Goa_campus_logo.gif",
    pilani:"https://www.bits-pilani.ac.in/Uploads/Campus/BITS_Pilani_campus_logo.gif",
    hyderabad:"https://www.bits-pilani.ac.in/Uploads/Campus/BITS_Hyderabad_campus_logo.gif"
}; 

document.body.style.backgroundImage = `url(${campusImages[campus]})`;
document.querySelector(".logo").setAttribute("src",campusLogos[campus]);
