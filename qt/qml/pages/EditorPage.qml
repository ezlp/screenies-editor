// EditorPage.qml — STUB. The SSRP editor is ported here in phase 2
// (photo load, crop, chatlog/text, live preview via compose::render, export).
import QtQuick
import QtQuick.Controls

Page {
    header: ToolBar {
        Row {
            anchors.fill: parent
            ToolButton { text: "← Menu"; onClicked: stack.pop() }
            Label { text: "SSRP Editor"; anchors.verticalCenter: parent.verticalCenter; leftPadding: 8 }
        }
    }
    Label {
        anchors.centerIn: parent
        horizontalAlignment: Text.AlignHCenter
        text: "SSRP Editor — coming in phase 2.\nPorts the current editor onto screenies-core."
        opacity: 0.7
    }
}
