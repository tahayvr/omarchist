// Omarchist's bar widget: the user's flows in a dropdown, and buttons that
// open Omarchist on a page. Installed and kept up to date by Omarchist
// itself (Settings > Bar), which writes the path of its binary into the
// `command` file next to this one.
import QtQuick
import QtQuick.Effects
import Quickshell
import Quickshell.Io
import qs.Commons
import qs.Ui

BarWidget {
    id: root
    moduleName: "tahayvr.omarchist"

    readonly property string icon: root.setting("icon", "󰏘")
    readonly property string pluginDir: Quickshell.env("HOME") + "/.config/omarchy/plugins/tahayvr.omarchist"
    // The binary Omarchist was started from; `omarchist` until the file loads.
    property string command: "omarchist"
    readonly property color fg: root.bar ? root.bar.foreground : Color.popups.text
    readonly property string face: root.bar && root.bar.fontFamily
                                   ? root.bar.fontFamily : Style.font.family
    readonly property string flowsDir: Quickshell.env("HOME") + "/.config/omarchist/flows"

    // [{ id, name, icon, steps }] from `omarchist flow list --json`
    property var flows: []
    readonly property var views: [
        { label: "Themes", view: "themes" },
        { label: "Configuration", view: "config" },
        { label: "Keybinds", view: "keybinds" },
        { label: "Flows", view: "flows" },
        { label: "Omarchy", view: "omarchy" },
        { label: "Settings", view: "settings" }
    ]

    // `omarchy-shell shell toggle tahayvr.omarchist` reaches these.
    readonly property bool opened: menu.open
    function open() { refresh(); menu.open = true; }
    function close() { menu.open = false; }
    function toggle() { if (menu.open) close(); else open(); }

    function refresh() { if (!listProc.running) listProc.running = true; }
    function runFlow(id) { close(); Util.execArgv([root.command, "flow", "run", id]); }
    function openView(view) { close(); Util.execArgv([root.command, "--view", view]); }

    implicitWidth: button.implicitWidth
    implicitHeight: barSize

    // Watched, so a moved binary (an update) is picked up without a restart.
    FileView {
        path: root.pluginDir + "/command"
        watchChanges: true
        printErrors: false
        onLoaded: {
            var line = text().trim();
            if (line !== "") root.command = line;
            root.refresh();
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
                    var parsed = JSON.parse(text);
                    root.flows = Array.isArray(parsed) ? parsed : [];
                } catch (e) {
                    root.flows = [];
                }
            }
        }
    }

    // Flows saved or removed in Omarchist show up without reopening.
    Process {
        running: true
        command: ["bash", "-c",
                  "mkdir -p \"$0\"; exec inotifywait -m -q -e close_write,create,delete,move --format %f \"$0\"",
                  root.flowsDir]
        stdout: SplitParser { onRead: function (_) { debounce.restart(); } }
    }
    Timer { id: debounce; interval: 200; onTriggered: root.refresh() }

    WidgetButton {
        id: button
        anchors.centerIn: parent
        bar: root.bar
        text: root.icon
        tooltipText: "Omarchist"
        onPressed: function (mouseButton) {
            if (mouseButton === Qt.RightButton) root.openView("flows");
            else root.toggle();
        }
    }

    PopupCard {
        id: menu
        anchorItem: button
        bar: root.bar
        contentWidth: menu.fittedContentWidth(Style.space(280))
        contentHeight: menu.fittedContentHeight(body.implicitHeight)

        Column {
            id: body
            width: parent.width
            spacing: Style.space(8)

            Text {
                text: "Flows"
                color: root.fg
                font.family: root.face
                font.pixelSize: Style.font.title
                font.bold: true
            }

            Text {
                visible: root.flows.length === 0
                text: "No flows yet"
                color: Qt.darker(root.fg, 1.4)
                font.family: root.face
                font.pixelSize: Style.font.body
            }

            Column {
                width: parent.width

                Repeater {
                    model: root.flows

                    Rectangle {
                        id: row
                        required property var modelData
                        width: parent.width
                        height: Style.spacing.popupRowHeight
                        radius: Style.cornerRadius
                        color: hover.hovered
                               ? Style.hoverFillFor(root.fg, Color.accent) : "transparent"

                        Row {
                            anchors.fill: parent
                            anchors.leftMargin: Style.space(8)
                            anchors.rightMargin: Style.space(8)
                            spacing: Style.space(10)

                            Item {
                                anchors.verticalCenter: parent.verticalCenter
                                width: Style.font.icon
                                height: Style.font.icon

                                Image {
                                    id: flowIcon
                                    anchors.fill: parent
                                    source: Qt.resolvedUrl("icons/" + row.modelData.icon + ".svg")
                                    sourceSize: Qt.size(width * 2, height * 2)
                                    fillMode: Image.PreserveAspectFit
                                    visible: false
                                }
                                MultiEffect {
                                    anchors.fill: flowIcon
                                    source: flowIcon
                                    colorization: 1.0
                                    colorizationColor: root.fg
                                }
                            }

                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                text: row.modelData.name
                                color: root.fg
                                font.family: root.face
                                font.pixelSize: Style.font.body
                                elide: Text.ElideRight
                            }
                        }

                        HoverHandler {
                            id: hover
                            cursorShape: Qt.PointingHandCursor
                        }
                        TapHandler {
                            acceptedButtons: Qt.LeftButton
                            onTapped: root.runFlow(row.modelData.id)
                        }
                        PanelToolTip {
                            visible: hover.hovered
                            text: "Run " + row.modelData.name
                        }
                    }
                }
            }

            PanelSeparator { foreground: root.fg }

            Text {
                text: "Open Omarchist"
                color: root.fg
                font.family: root.face
                font.pixelSize: Style.font.title
                font.bold: true
            }

            Flow {
                width: parent.width
                spacing: Style.space(6)

                Repeater {
                    model: root.views

                    Button {
                        required property var modelData
                        text: modelData.label
                        foreground: root.fg
                        fontFamily: root.face
                        onClicked: root.openView(modelData.view)
                    }
                }
            }
        }
    }
}
