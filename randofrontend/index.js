// Note that a dynamic `import` statement here is required due to
// webpack/webpack#6615, but in theory `import { greet } from './pkg';`
// will work here one day as well!
const rust = import('./pkg');

function randomizeRom(module) {
    let seed = document.getElementById('seed').value;
    let file = document.getElementById('rom').files;

    if(file.length === 0) {
        console.log("Couldn't find file to randomize");
        return;
    } else {
        console.log(file);
    }

    let reader = new FileReader();
    reader.onload = function() {

        const arrayBuffer = this.result,
            array = new Uint8Array(arrayBuffer),
            binaryString = String.fromCharCode.apply(null, array);

        //console.log(binaryString);
        let randomizedRom = module.randomize_rom(array, seed);
        console.log(randomizedRom);

        var blob = new Blob([randomizedRom.get_rom()], {type: "application/octet-stream" });
        var URL = window.URL || window.webkitURL;
        var downloadUrl = URL.createObjectURL(blob);
        var a = document.createElement("a");
        a.href = downloadUrl;
        a.download = randomizedRom.get_filename();
        document.body.appendChild(a);
        a.click();
        setTimeout(function () { URL.revokeObjectURL(downloadUrl); }, 100);
    }
    reader.readAsArrayBuffer(file[0]);
}


rust
    .then(m => {
        document.getElementById('rom').addEventListener("change", randomizeRom(m));
    })
    .catch(console.error);