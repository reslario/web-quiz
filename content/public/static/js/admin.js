const questionForm = "questionForm";
let questionTable;

async function fillForms() {
    setUpForms();
    const categories = await fetch("/admin/all_categories", { credentials: "include" })
        .then(resp => resp.json())
        .then(json => json.categories);
    fillCategories(categories);
    await fillTable(categories);
}

function fillCategories(categories) {
    const select = document.getElementById("availableCategories");
    categories
        .map(cat => {
            const option = document.createElement("option");
            option.value = cat.id;
            option.innerText = cat.name;
            return option;
        })
        .forEach(option => select.add(option))
}

async function fillTable(categories) {
    questionTable = document.getElementById("questions");
    await fetch("/admin/all_questions", { credentials: "include" })
        .then(async resp => await resp.json())
        .then(resp => resp.questions.forEach(q =>
            questionTable.appendChild(tableRow(q, categories))
        ));
}

function setUpForms() {
    const editQuestionForm = document.getElementById(questionForm);
    editQuestionForm.onsubmit = async (e) => await submit(e, editQuestionForm);
}

function tableRow(question, categories) {
    const row = document.createElement("tr");
    const addCell = (elm) => row.insertCell(-1).appendChild(elm);
    const addTextCell = (name, val) => addCell(textField(name, val));

    let id = addTextCell("id", question.id);
    id.readOnly = true;
    addCell(categorySelect(question, categories));
    addTextCell("string", question.string);
    addTextCell("correct", question.correct);
    addTextCell("incorrect1", question.incorrect[0]);
    addTextCell("incorrect2", question.incorrect[1]);
    addTextCell("incorrect3", question.incorrect[2]);
    addCell(editButton());
    addCell(deleteButton());

    return row;
}

function textField(name, val) {
    const elm = document.createElement("input");
    elm.setAttribute("form", questionForm);
    elm.type = "text";
    elm.name = name;
    elm.value = val;
    return elm;
}

const editButton = () => button("edit", "Bearbeiten");

const deleteButton = () => button("delete", "Löschen");

function button(action, text) {
    const btn = document.createElement("button");
    btn.setAttribute("form", questionForm);
    btn.type = "submit";
    btn.formAction = action;
    btn.innerText = text;
    return btn;
}

function categorySelect(question, categories) {
    let select = document.createElement("select");
    select.name = "category";
    select.classList.add("categories");
    categories.forEach(cat => {
        let option = document.createElement("option");
        option.value = cat.id;
        option.innerText = cat.name;
        if (question.category_id === cat.id) {
            option.selected = true;
        }
        select.appendChild(option);
    });

    return select;
}

async function submit(event, form) {
    event.preventDefault();
    const action = event
        .explicitOriginalTarget
        .attributes
        .formaction
        .value;

    const row = event
        .explicitOriginalTarget
        .parentNode
        .parentNode;

    const values = Array.from(row.childNodes)
        .map(td => td.firstChild)
        .filter(input => input.tagName !== "BUTTON")
        .reduce((map, tf) => {
            map[tf.name] = tf.value;
            return map;
        }, {});

    switch (action) {
        case "edit":
            await editQ(values);
            break;
        case "delete":
            await deleteQ(values, row);
            break;
    }
}

async function editQ(values) {
    let obj = {
        id: parseInt(values.id),
        category_id: parseInt(values.category),
        string: values.string,
        correct: values.correct,
        incorrect: [values.incorrect1, values.incorrect2, values.incorrect3]
    };

    await fetch(
        "/admin/edit_question",
        { credentials: "include", method: "put", body: JSON.stringify(obj)}
    );
}

async function deleteQ(values, row) {
    let resp = await fetch(
        `/admin/delete_question/${values.id}`,
        { method: "delete", credentials: "include"}
    );

    if (resp.ok) {
        questionTable.removeChild(row)
    }
}


