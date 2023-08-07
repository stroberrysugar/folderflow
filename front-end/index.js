window.addEventListener('load', () => {
    load().then(() => { }).catch(err => {
        alert(`Error loading: ${err}`);
    });
});

async function load() {
    await loadFolders();
    await loadScripts();

}

async function loadScripts() {
    const response = await fetch(`/api/list_scripts`, {
        method: 'GET'
    });

    let responseText = await response.text();

    if (!response.ok) {
        throw responseText;
    }

    const scriptFolderChoiceSelect = document.getElementById("scriptFolderChoice");
    const scriptChoiceSelect = document.getElementById("scriptChoice");
    let scriptList = JSON.parse(responseText);
    let scriptListArray = Array.from(scriptList);

    if (scriptListArray.length == 0) {
        document.getElementById("executeScriptButton").setAttribute("disabled", "disabled");
        return;
    }

    scriptListArray.forEach(script => {
        let option = document.createElement("option");

        option.innerText = `${script.friendly_name}`;

        option.execute = async () => {
            let folderName = scriptFolderChoiceSelect.options[scriptFolderChoiceSelect.options.selectedIndex].getName();

            const response = await fetch(`/api/execute?script_id=${script.id}&folder_name=${encodeURIComponent(folderName)}`, {
                method: 'POST'
            });

            let responseText = await response.text();

            if (!response.ok) {
                console.error(`Error executing script: ${responseText}`);
                alert(`Error executing script: ${responseText}`);

                return;
            }

            let executionComplete = JSON.parse(responseText);

            const commandOutputExitStatus = document.getElementById("commandOutputExitStatus");
            const commandOutputStdout = document.getElementById("commandOutputStdout");
            const commandOutputStderr = document.getElementById("commandOutputStderr");

            commandOutputExitStatus.innerText = executionComplete.exit_status;
            commandOutputStdout.innerText = executionComplete.stdout ? executionComplete.stdout : " ";
            commandOutputStderr.innerText = executionComplete.stderr ? executionComplete.stderr : " ";
        };

        option.update = () => {
            let folderPath = scriptFolderChoiceSelect.options[scriptFolderChoiceSelect.options.selectedIndex].getPath();
            document.getElementById("commandRun").innerText = `${script.path} "${folderPath}"`;
        };

        scriptChoiceSelect.append(option);
    });

    if (scriptFolderChoiceSelect.options.length != 0) {
        updateCommandRun();
    }
}

async function loadFolders() {
    const response = await fetch(`/api/list_folders`, {
        method: 'GET'
    });

    let responseText = await response.text();

    if (!response.ok) {
        throw responseText;
    }

    let folderChoiceSelect = document.getElementById("folderChoice");
    let scriptFolderChoiceSelect = document.getElementById("scriptFolderChoice");
    let folderList = JSON.parse(responseText);
    let folderListArray = Array.from(folderList);

    folderListArray.sort((a, b) => new Date(b.created_at) - new Date(a.created_at));

    if (folderListArray.length == 0) {
        document.getElementById("downloadFolderButton").setAttribute("disabled", "disabled");
        document.getElementById("executeScriptButton").setAttribute("disabled", "disabled");
        return;
    }

    folderListArray.forEach(folder => {
        let option1 = document.createElement("option");

        option1.innerText = `${new Date(folder.created_at).toLocaleString()} | ${folder.name}`;

        option1.download = () => {
            triggerDownload(`/api/download?folder_name=${encodeURIComponent(folder.name)}`);
        };

        let option2 = document.createElement("option");
        option2.innerText = `${new Date(folder.created_at).toLocaleString()} | ${folder.name}`;
        option2.getName = () => {
            return folder.name;
        };
        option2.getPath = () => {
            return folder.full_path;
        };

        folderChoiceSelect.append(option1);
        scriptFolderChoiceSelect.append(option2);
    });
}

async function executeScript() {
    const executeScriptButton = document.getElementById("executeScriptButton");
    const scriptChoice = document.getElementById("scriptChoice");

    let scriptOption = scriptChoice.options[scriptChoice.options.selectedIndex];

    executeScriptButton.setAttribute("disabled", "disabled");
    await scriptOption.execute();
    executeScriptButton.removeAttribute("disabled");
}

async function updateCommandRun() {
    document
        .getElementById("scriptChoice")
        .options[scriptChoice.options.selectedIndex]
        .update();
}

async function downloadFolderContents() {
    const downloadFolderButton = document.getElementById("downloadFolderButton");
    const folderChoice = document.getElementById("folderChoice");

    let option = folderChoice.options[folderChoice.options.selectedIndex];

    downloadFolderButton.setAttribute("disabled", "disabled");
    option.download();
    downloadFolderButton.removeAttribute("disabled");
}

async function uploadFolderContents() {
    const input = document.getElementById('fileInput');
    const folderName = document.getElementById("folderName");

    folderName.reportValidity();

    const formData = new FormData();
    const files = input.files;

    if (files.length == 0) {
        alert("Please select some files to upload");
        return;
    }

    input.setAttribute("disabled", "disabled");
    folderName.setAttribute("disabled", "disabled");

    for (let i = 0; i < files.length; i++) {
        formData.append(files[i].name, files[i]);
    }

    let queryArgs = folderName.value ? `?folder_name=${folderName.value}` : "";

    const response = await fetch(`/api/upload${queryArgs}`, {
        method: 'POST',
        body: formData
    });

    if (response.ok) {
        console.log('Folder contents uploaded successfully');
    } else {
        let error = await response.text();
        console.error(`Error uploading folder contents: ${error}`);
        alert(`Error uploading folder contents: ${error}`);
    }

    input.value = null;
    folderName.value = null;

    input.removeAttribute("disabled");
    folderName.removeAttribute("disabled");

    location.reload();
}

function triggerDownload(url) {
    const link = document.createElement('a');
    link.href = url;
    link.target = '_blank'; // Open in a new tab/window
    link.download = ''; // This attribute triggers the download behavior
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
}
