// Omarchist's bar widget: a row of buttons that open Omarchist on a page,
// the user's flows to run, and the installed version. Installed and kept up to date by Omarchist
// itself (Settings > Bar), which writes the path of its binary into the
// `command` file next to this one.
//
// Every icon is a font glyph (Material Design Nerd Font, and the `omarchy`
// font's logo) so it takes the bar's theme colors like Omarchy's own panels.
import QtQuick
import QtQuick.Controls
import Quickshell
import Quickshell.Io
import qs.Commons
import qs.Ui

Panel {
    id: root
    moduleName: "tahayvr.omarchist"
    ipcTarget: "tahayvr.omarchist"

    readonly property string icon: root.setting("icon", "\u{f0843}")
    readonly property string pluginDir: Quickshell.env("HOME") + "/.config/omarchy/plugins/tahayvr.omarchist"
    readonly property string flowsDir: Quickshell.env("HOME") + "/.config/omarchist/flows"
    // The binary Omarchist was started from; `omarchist` until the file loads.
    property string command: "omarchist"

    readonly property color fg: root.bar ? root.bar.foreground : Color.foreground
    readonly property color dim: Qt.darker(root.fg, 1.4)
    readonly property string face: root.bar && root.bar.fontFamily ? root.bar.fontFamily : Style.font.family

    // "2.0.0" from `omarchist --version`.
    property string version: ""
    // [{ id, name, icon, glyph, steps }] from `omarchist flow list --json`
    property var flows: []
    readonly property var views: [
        { label: "Themes", view: "themes", glyph: "\u{f03d8}", font: "" },
        { label: "Configuration", view: "config", glyph: "\u{f0493}", font: "" },
        { label: "Keybinds", view: "keybinds", glyph: "\u{f030c}", font: "" },
        { label: "Flows", view: "flows", glyph: "\u{f04aa}", font: "" },
        { label: "Omarchy", view: "omarchy", glyph: "\ue900", font: "omarchy" },
        { label: "Settings", view: "settings", glyph: "\u{f1542}", font: "" }
    ]

    // One keyboard cursor over both sections; the mouse moves it too, so
    // only one item is ever highlighted.
    property bool cursorActive: false
    property bool keyboardCursor: false
    property string section: "views"
    property int viewIndex: 0
    property int flowIndex: 0

    function refresh() {
        if (!listProc.running) listProc.running = true
        if (!versionProc.running) versionProc.running = true
    }
    function runFlow(id) { close(); Util.execArgv([root.command, "flow", "run", id]) }
    function openView(view) {
        close()
        Util.execArgv(view === "" ? [root.command] : [root.command, "--view", view])
    }

    function moveCursor(dx, dy) {
        root.keyboardCursor = true
        if (!root.cursorActive) { root.cursorActive = true; return }
        if (root.section === "views") {
            if (dx !== 0) root.viewIndex = Math.max(0, Math.min(root.views.length - 1, root.viewIndex + dx))
            if (dy > 0 && root.flows.length > 0) { root.section = "flows"; root.flowIndex = 0 }
        } else {
            var next = root.flowIndex + dy
            if (next < 0) root.section = "views"
            else root.flowIndex = Math.min(root.flows.length - 1, next)
        }
        scrollToCursor()
    }

    function activateCursor() {
        if (!root.cursorActive) return
        if (root.section === "views") root.openView(root.views[root.viewIndex].view)
        else if (root.flows[root.flowIndex]) root.runFlow(root.flows[root.flowIndex].id)
    }

    function hoverView(index) {
        root.keyboardCursor = false
        root.cursorActive = true
        root.section = "views"
        root.viewIndex = index
    }

    function hoverFlow(index) {
        root.keyboardCursor = false
        root.cursorActive = true
        root.section = "flows"
        root.flowIndex = index
    }

    function scrollToCursor() {
        if (root.section !== "flows") { flick.contentY = 0; return }
        var row = flowRepeater.itemAt(root.flowIndex)
        if (!row) return
        var y = row.mapToItem(column, 0, 0).y
        if (y < flick.contentY) flick.contentY = y
        else if (y + row.height > flick.contentY + flick.height)
            flick.contentY = y + row.height - flick.height
    }

    onOpenedChanged: {
        if (!opened) return
        refresh()
        root.cursorActive = false
        root.keyboardCursor = false
        root.section = "views"
        root.viewIndex = 0
        root.flowIndex = 0
        flick.contentY = 0
    }

    implicitWidth: button.implicitWidth
    implicitHeight: button.implicitHeight

    // Watched, so a moved binary (an update) is picked up without a restart.
    FileView {
        path: root.pluginDir + "/command"
        watchChanges: true
        printErrors: false
        onLoaded: {
            var line = text().trim()
            if (line !== "") root.command = line
            root.refresh()
        }
        onFileChanged: reload()
    }

    Process {
        id: listProc
        command: [root.command, "flow", "list", "--json"]
        stdout: StdioCollector {
            waitForEnd: true
            onStreamFinished: {
                try {
                    var parsed = JSON.parse(text)
                    root.flows = Array.isArray(parsed) ? parsed : []
                } catch (e) {
                    root.flows = []
                }
                if (root.flowIndex >= root.flows.length) root.flowIndex = Math.max(0, root.flows.length - 1)
                if (root.flows.length === 0 && root.section === "flows") root.section = "views"
            }
        }
    }

    Process {
        id: versionProc
        command: [root.command, "--version"]
        stdout: StdioCollector {
            waitForEnd: true
            onStreamFinished: {
                var words = text.trim().split(/\s+/)
                root.version = words.length > 1 ? "Omarchist " + words[words.length - 1] : ""
            }
        }
    }

    // Flows saved or removed in Omarchist show up without reopening.
    Process {
        running: true
        command: ["bash", "-c",
                  "mkdir -p \"$0\"; exec inotifywait -m -q -e close_write,create,delete,move --format %f \"$0\"",
                  root.flowsDir]
        stdout: SplitParser { onRead: function (_) { debounce.restart() } }
    }
    Timer { id: debounce; interval: 200; onTriggered: root.refresh() }

    BarIconButton {
        id: button
        anchors.fill: parent
        bar: root.bar
        text: root.icon
        tooltipText: root.opened ? "" : "Omarchist"
        onPressed: function (mouseButton) {
            if (mouseButton === Qt.RightButton) root.openView("")
            else root.toggle()
        }
    }

    KeyboardPanel {
        id: panel
        anchorItem: button
        owner: root
        bar: root.bar
        open: root.opened
        focusTarget: keyCatcher
        contentWidth: panel.fittedContentWidth(Style.space(340))
        contentHeight: panel.fittedContentHeight(column.implicitHeight, Style.space(520))

        PanelKeyCatcher {
            id: keyCatcher
            anchors.fill: parent
            onMoveRequested: function (dx, dy) { root.moveCursor(dx, dy) }
            onActivateRequested: root.activateCursor()
            onCloseRequested: root.close()
            onTabRequested: function (direction) { root.switchPanel(direction) }

            Flickable {
                id: flick
                anchors.fill: parent
                contentWidth: width
                contentHeight: column.implicitHeight
                clip: true
                boundsBehavior: Flickable.StopAtBounds
                interactive: contentHeight > height
                ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }

                Column {
                    id: column
                    width: flick.width
                    spacing: Style.space(14)

                    // ---------- Open a page ----------
                    Row {
                        id: viewRow
                        width: parent.width
                        spacing: Style.space(6)
                        readonly property real cellWidth: (width - spacing * (root.views.length - 1)) / root.views.length

                        Repeater {
                            model: root.views

                            CursorSurface {
                                id: cell
                                required property var modelData
                                required property int index
                                readonly property bool hasFocus: root.cursorActive && root.section === "views"
                                                                 && root.viewIndex === index

                                width: viewRow.cellWidth
                                height: Style.space(44)
                                bordered: true
                                hasCursor: hasFocus
                                foreground: root.fg

                                Text {
                                    anchors.centerIn: parent
                                    textFormat: Text.PlainText
                                    text: cell.modelData.glyph
                                    color: root.fg
                                    font.family: cell.modelData.font !== "" ? cell.modelData.font : root.face
                                    font.pixelSize: Math.round(Style.font.title * 1.35)
                                }

                                MouseArea {
                                    id: cellMouse
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    cursorShape: Qt.PointingHandCursor
                                    onContainsMouseChanged: if (containsMouse) root.hoverView(cell.index)
                                    onClicked: root.openView(cell.modelData.view)
                                }

                                PanelToolTip {
                                    visible: cellMouse.containsMouse || (cell.hasFocus && root.keyboardCursor)
                                    text: cell.modelData.label
                                    fontFamily: root.face
                                }
                            }
                        }
                    }

                    PanelSeparator { foreground: root.fg }

                    // ---------- Flows ----------
                    Column {
                        width: parent.width
                        spacing: Style.space(6)

                        PanelSectionHeader {
                            text: "FLOWS"
                            foreground: root.fg
                            fontFamily: root.face
                        }

                        Text {
                            visible: root.flows.length === 0
                            width: parent.width
                            textFormat: Text.PlainText
                            text: "Create one on the Flows page."
                            color: root.dim
                            font.family: root.face
                            font.pixelSize: Style.font.bodySmall
                        }

                        Column {
                            width: parent.width
                            spacing: Style.space(2)

                            Repeater {
                                id: flowRepeater
                                model: root.flows

                                CursorSurface {
                                    id: row
                                    required property var modelData
                                    required property int index
                                    readonly property bool hasFocus: root.cursorActive && root.section === "flows"
                                                                     && root.flowIndex === index

                                    width: parent.width
                                    height: Style.space(40)
                                    hasCursor: hasFocus
                                    foreground: root.fg

                                    Text {
                                        id: flowGlyph
                                        anchors.left: parent.left
                                        anchors.leftMargin: Style.space(10)
                                        anchors.verticalCenter: parent.verticalCenter
                                        width: Style.font.title + Style.space(4)
                                        horizontalAlignment: Text.AlignHCenter
                                        textFormat: Text.PlainText
                                        text: row.modelData.glyph || "\u{f04aa}"
                                        color: root.fg
                                        font.family: root.face
                                        font.pixelSize: Style.font.title
                                    }

                                    Column {
                                        anchors.left: flowGlyph.right
                                        anchors.leftMargin: Style.space(10)
                                        anchors.right: runGlyph.left
                                        anchors.rightMargin: Style.space(8)
                                        anchors.verticalCenter: parent.verticalCenter
                                        spacing: Style.space(1)

                                        Text {
                                            width: parent.width
                                            textFormat: Text.PlainText
                                            text: row.modelData.name
                                            color: root.fg
                                            font.family: root.face
                                            font.pixelSize: Style.font.body
                                            elide: Text.ElideRight
                                        }

                                        Text {
                                            width: parent.width
                                            textFormat: Text.PlainText
                                            text: (row.modelData.steps === 1 ? "1 step" : row.modelData.steps + " steps").toUpperCase()
                                            color: root.dim
                                            font.family: root.face
                                            font.pixelSize: Style.font.caption
                                            font.bold: true
                                            font.letterSpacing: 1.2
                                        }
                                    }

                                    Text {
                                        id: runGlyph
                                        anchors.right: parent.right
                                        anchors.rightMargin: Style.space(12)
                                        anchors.verticalCenter: parent.verticalCenter
                                        textFormat: Text.PlainText
                                        text: "\u{f040a}"
                                        color: root.fg
                                        opacity: row.hasFocus ? 1 : 0
                                        font.family: root.face
                                        font.pixelSize: Style.font.title
                                        Behavior on opacity { NumberAnimation { duration: 80 } }
                                    }

                                    MouseArea {
                                        id: rowMouse
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onContainsMouseChanged: if (containsMouse) root.hoverFlow(row.index)
                                        onClicked: root.runFlow(row.modelData.id)
                                    }

                                    PanelToolTip {
                                        visible: rowMouse.containsMouse
                                        text: "Run " + row.modelData.name
                                        fontFamily: root.face
                                    }
                                }
                            }
                        }
                    }

                    PanelSeparator {
                        visible: root.version !== ""
                        foreground: root.fg
                    }

                    Text {
                        visible: root.version !== ""
                        width: parent.width
                        textFormat: Text.PlainText
                        text: root.version.toUpperCase()
                        color: root.dim
                        font.family: root.face
                        font.pixelSize: Style.font.caption
                        font.bold: true
                        font.letterSpacing: 1.2
                    }
                }
            }
        }
    }
}
