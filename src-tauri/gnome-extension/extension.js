import Gio from 'gi://Gio';
import Meta from 'gi://Meta';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

const NAME = 'io.github.enesdurmus.EndSnap';
const PATH = '/io/github/enesdurmus/EndSnap';
const XML = `<node><interface name="${NAME}">
  <method name="Ping"/>
  <method name="Info"><arg type="a(iiii)" direction="out"/></method>
  <method name="MoveResize">
    <arg type="i" direction="in"/><arg type="i" direction="in"/>
    <arg type="i" direction="in"/><arg type="i" direction="in"/>
  </method>
</interface></node>`;

const rect = r => [r.x, r.y, r.width, r.height];

export default class EndSnap extends Extension {
    enable() {
        this._dbus = Gio.DBusExportedObject.wrapJSObject(XML, this);
        this._dbus.export(Gio.DBus.session, PATH);
        this._owner = Gio.bus_own_name_on_connection(Gio.DBus.session, NAME, Gio.BusNameOwnerFlags.NONE, null, null);
    }

    disable() {
        Gio.bus_unown_name(this._owner);
        this._dbus.unexport();
        this._dbus = null;
    }

    _window() {
        const win = global.display.focus_window;
        if (!win || win.get_window_type() !== Meta.WindowType.NORMAL)
            throw new Error('no window to snap');
        return win;
    }

    // lets End check the extension is up, even with nothing focused
    Ping() {}

    // [focused window frame, work area of every monitor]
    Info() {
        const areas = [];
        for (let i = 0; i < global.display.get_n_monitors(); i++)
            areas.push(rect(Main.layoutManager.getWorkAreaForMonitor(i)));
        return [rect(this._window().get_frame_rect()), ...areas];
    }

    MoveResize(x, y, w, h) {
        const win = this._window();
        // a maximized or tiled window ignores move_resize_frame
        try { win.unmaximize(); } catch { win.unmaximize(Meta.MaximizeFlags.BOTH); }
        win.move_resize_frame(true, x, y, w, h);
    }
}
