// SkateBIRD lab plugin: scene loads, teleports and an exact camera, driven through a control folder.
//
// The control folder is <game root>/control (the lab copy's folder, BepInEx's Paths.GameRootPath). At start the plugin
// deletes command.json and reply.json, then writes ready.json ({"pid": <game pid>}); after that a runner writes
// command.json atomically (write a temp file, rename); the plugin runs each new seq once and writes reply.json the
// same way, with the same seq. Ops (JsonUtility fields, missing ones keep their defaults):
//
//   {"seq":1,"op":"load","scene":"rooftop"}              LoadingManager.LoadScene(Unity scene name); replies once
//                                                        the scene is active, loading is over and the player exists
//   {"seq":2,"op":"screen","width":1024,"height":576}    Screen.SetResolution(w, h, windowed); replies when applied
//   {"seq":3,"op":"teleport","position":[x,y,z],"yaw":172.9}
//                                                        puts the skateboard at a Unity position and yaw
//   {"seq":4,"op":"camera","position":[x,y,z],"yaw":172.9,"pitch":0,"fov":60,"settle_frames":30,"settle_seconds":1}
//                                                        locks Camera.main at that world pose: rotation
//                                                        Euler(pitch, yaw, 0) (pitch > 0 looks down, yaw from +Z
//                                                        towards +X), vertical field of view fov; CinemachineBrain
//                                                        off; screen/camera-space canvases and the player's
//                                                        renderers hidden (unless "show_player":true); depth of
//                                                        field, lens distortion, chromatic aberration, vignette
//                                                        and motion blur off (unless "keep_post":true); with
//                                                        "flat_sky":true the sky is cleared to black, no skybox;
//                                                        re-asserted every frame until release; replies with the
//                                                        read-back pose after the settle
//   {"seq":5,"op":"shot","path":"C:/out/0_spawn.png"}    reads the rendered frame back at the end of a frame, writes
//                                                        it as PNG and replies with the camera pose of that frame
//   {"seq":6,"op":"readback"}                            replies with the current pose
//   {"seq":7,"op":"release"}                             gives the camera, canvases and renderers back
//   {"seq":8,"op":"quit"}                                Application.Quit
//
// Every reply carries "ok" and, on failure, "error"; pose replies carry the camera's position, euler angles, forward,
// bearing (atan2(fwd.x, fwd.z)), fieldOfView and the vertical FOV of the projection matrix, the screen size, the
// player's position, what was hidden, the colliders within 0.2 m of the camera (cam_overlaps) and the distance down
// to the first collider below it (cam_ground). Everything is also logged to BepInEx/LogOutput.log.
using System;
using System.Collections;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using BepInEx;
using Cinemachine;
using Skatebirb;
using Skatebirb.Managers;
using Skatebirb.Util;
using UnityEngine;
using UnityEngine.Rendering.PostProcessing;
using UnityEngine.SceneManagement;

namespace SkateBIRDTeleport
{
    [Serializable]
    public class Command
    {
        public int seq;
        public string op;
        public string scene;
        public string path;
        public float[] position;
        public float yaw;
        public float pitch;
        public float fov = 60f;
        public int settle_frames = 30;
        public float settle_seconds = 1f;
        public bool show_player;
        public bool keep_post;
        public bool flat_sky;
        public int width;
        public int height;
    }

    [Serializable]
    public class Reply
    {
        public int seq;
        public string op;
        public bool ok;
        public string error;
        public string scene;
        public float time;
        public int frame;
        public string path;
        public bool locked;
        public float[] cam_position;
        public float[] cam_euler;
        public float[] cam_forward;
        public float cam_bearing;
        public float cam_fov;
        public float cam_fov_projection;
        public float cam_near;
        public float cam_far;
        public string cam_name;
        public int screen_width;
        public int screen_height;
        public float[] player_position;
        public float player_yaw;
        public string[] cam_overlaps;
        public float cam_ground;
        public bool gameplay_obstructed;
        public int hidden_canvases;
        public int hidden_renderers;
        public string[] player_roots;
        public string[] other_cameras;
        public string[] kept_canvases;
        public string[] post_effects_off;
        public string[] post_effects_on;
    }

    [BepInPlugin("skatebird.teleport", "SkateBIRD lab control", "0.5.0")]
    public class TeleportPlugin : BaseUnityPlugin
    {
        private const float PollSeconds = 0.1f;
        // A player root with more renderers than this is not the player (a scene container); it is left alone.
        private const int MaxPlayerRenderers = 400;
        private const float CamProbe = 0.2f;

        private string controlDir;
        private string commandPath;
        private string replyPath;
        private float nextPoll;
        private int lastSeq = int.MinValue;
        private bool busy;

        private Camera lockCamera;
        private CinemachineBrain lockBrain;
        private Vector3 lockPosition;
        private Quaternion lockRotation;
        private float lockFov;
        private readonly List<Canvas> hiddenCanvases = new List<Canvas>();
        private readonly HashSet<Renderer> hiddenRenderers = new HashSet<Renderer>();
        private readonly List<string> playerRoots = new List<string>();
        private bool showPlayer;
        private bool keepPost;
        private bool flatSky;
        private CameraClearFlags savedClearFlags;
        private Color savedBackground;
        // Effects that make a frame differ from a pinhole render of the scene (blur, lens warp, colour fringes, dark
        // corners). The game's own options (Options DisableDoF/DisableVignette) do the same but persist to the save.
        private static readonly Type[] PostEffectsOff =
            { typeof(DepthOfField), typeof(LensDistortion), typeof(ChromaticAberration), typeof(Vignette), typeof(MotionBlur) };
        private readonly HashSet<PostProcessEffectSettings> postOff = new HashSet<PostProcessEffectSettings>();

        private void Awake()
        {
            controlDir = Path.Combine(Paths.GameRootPath, "control");
            Directory.CreateDirectory(controlDir);
            commandPath = Path.Combine(controlDir, "command.json");
            replyPath = Path.Combine(controlDir, "reply.json");
            // A stale command from an earlier run must not replay (the runner starts every run at seq 1), so the
            // runner sends nothing before it has read ready.json with this process's id.
            TryDelete(commandPath);
            TryDelete(replyPath);
            var ready = Path.Combine(controlDir, "ready.json");
            File.WriteAllText(ready + ".tmp", "{\"pid\":" + System.Diagnostics.Process.GetCurrentProcess().Id + "}");
            ReplaceFile(ready + ".tmp", ready);
            // Keep rendering and reading commands while the window is not focused.
            Application.runInBackground = true;
            Camera.onPreCull += OnAnyPreCull;
            SceneManager.sceneLoaded += (s, m) => Logger.LogInfo("SCENE ACTIVE " + SceneManager.GetActiveScene().name);
            Logger.LogInfo("CONTROL " + controlDir);
        }

        private void OnDestroy()
        {
            Camera.onPreCull -= OnAnyPreCull;
        }

        private void Update()
        {
            if (lockCamera != null) HoldLock();
            if (busy || Time.unscaledTime < nextPoll) return;
            nextPoll = Time.unscaledTime + PollSeconds;
            Poll();
        }

        private void LateUpdate()
        {
            if (lockCamera != null) HoldLock();
        }

        private void OnAnyPreCull(Camera cam)
        {
            if (cam == null || cam != lockCamera) return;
            // Last word before this camera culls: no game script can move it or show the player after this.
            ApplyPose();
            HidePlayer();
        }

        private void Poll()
        {
            string text;
            try
            {
                if (!File.Exists(commandPath)) return;
                text = File.ReadAllText(commandPath);
            }
            catch (IOException)
            {
                return;
            }
            Command cmd;
            try
            {
                cmd = JsonUtility.FromJson<Command>(text);
            }
            catch (Exception e)
            {
                Logger.LogError("COMMAND BAD JSON: " + e.Message);
                return;
            }
            if (cmd == null || cmd.seq == lastSeq) return;
            lastSeq = cmd.seq;
            Logger.LogInfo("COMMAND " + text.Trim());
            busy = true;
            StartCoroutine(Run(cmd));
        }

        private IEnumerator Run(Command cmd)
        {
            IEnumerator body;
            switch (cmd.op)
            {
                case "load": body = DoLoad(cmd); break;
                case "screen": body = DoScreen(cmd); break;
                case "teleport": body = DoTeleport(cmd); break;
                case "camera": body = DoCamera(cmd); break;
                case "shot": body = DoShot(cmd); break;
                case "readback": body = DoReadback(cmd); break;
                case "release": body = DoRelease(cmd); break;
                case "quit": body = DoQuit(cmd); break;
                default: body = DoUnknown(cmd); break;
            }
            // Nested iteration instead of StartCoroutine so an exception in an op becomes a failed reply; the finally
            // frees the command loop whatever happens (a stuck busy flag ignores every later command).
            try
            {
                while (true)
                {
                    object current;
                    try
                    {
                        if (!body.MoveNext()) break;
                        current = body.Current;
                    }
                    catch (Exception e)
                    {
                        Logger.LogError("OP " + cmd.op + " THREW " + e);
                        Fail(cmd, e.GetType().Name + ": " + e.Message);
                        break;
                    }
                    yield return current;
                }
            }
            finally
            {
                busy = false;
            }
        }

        private IEnumerator DoUnknown(Command cmd)
        {
            Fail(cmd, "unknown op '" + cmd.op + "'");
            yield break;
        }

        // ---- ops -------------------------------------------------------------------------------------------------

        private IEnumerator DoLoad(Command cmd)
        {
            if (string.IsNullOrEmpty(cmd.scene))
            {
                Fail(cmd, "load needs scene");
                yield break;
            }
            ReleaseLock();
            var deadline = Time.unscaledTime + 30f;
            while (LoadingManager.IsLoading)
            {
                if (Time.unscaledTime > deadline) { Fail(cmd, "a load already in progress did not finish"); yield break; }
                yield return null;
            }
            LoadingManager.LoadScene(cmd.scene);
            deadline = Time.unscaledTime + 120f;
            while (LoadingManager.IsLoading || SceneManager.GetActiveScene().name != cmd.scene
                   || ImportantStuff.PlayerSkateboard == null || Camera.main == null)
            {
                if (Time.unscaledTime > deadline)
                {
                    Fail(cmd, "scene '" + cmd.scene + "' not active after 120 s (active: "
                              + SceneManager.GetActiveScene().name + ")");
                    yield break;
                }
                yield return null;
            }
            // ControlBoard resets the player to the level start on its first Update: let that and the fade-in run.
            var until = Time.unscaledTime + Math.Max(cmd.settle_seconds, 0f);
            for (var i = 0; i < cmd.settle_frames || Time.unscaledTime < until; i++) yield return null;
            Succeed(cmd, Pose(cmd));
        }

        private IEnumerator DoScreen(Command cmd)
        {
            if (cmd.width <= 0 || cmd.height <= 0)
            {
                Fail(cmd, "screen needs width and height");
                yield break;
            }
            if (Screen.width != cmd.width || Screen.height != cmd.height || Screen.fullScreen)
                Screen.SetResolution(cmd.width, cmd.height, FullScreenMode.Windowed);
            var deadline = Time.unscaledTime + 10f;
            while (Screen.width != cmd.width || Screen.height != cmd.height)
            {
                if (Time.unscaledTime > deadline)
                {
                    Fail(cmd, Format("screen is {0}x{1}, not {2}x{3}", Screen.width, Screen.height, cmd.width, cmd.height));
                    yield break;
                }
                yield return null;
            }
            Succeed(cmd, Pose(cmd));
        }

        private IEnumerator DoTeleport(Command cmd)
        {
            Vector3 pos;
            if (!TryVector(cmd.position, out pos)) { Fail(cmd, "teleport needs position [x,y,z]"); yield break; }
            var board = ImportantStuff.PlayerSkateboard;
            if (board == null) { Fail(cmd, "no player (ImportantStuff.PlayerSkateboard is null)"); yield break; }
            var rot = Quaternion.Euler(0f, cmd.yaw, 0f);
            // Hold through six physics steps: a moving board would otherwise be snapped back.
            for (var i = 0; i < 6; i++)
            {
                board.Teleport(pos, rot);
                var rb = board.RigidBody;
                rb.velocity = Vector3.zero;
                rb.angularVelocity = Vector3.zero;
                rb.position = pos;
                rb.rotation = rot;
                yield return new WaitForFixedUpdate();
            }
            board.Teleport(pos, rot);
            yield return null;
            Succeed(cmd, Pose(cmd));
        }

        private IEnumerator DoCamera(Command cmd)
        {
            Vector3 pos;
            if (!TryVector(cmd.position, out pos)) { Fail(cmd, "camera needs position [x,y,z]"); yield break; }
            if (!(cmd.fov > 1f && cmd.fov < 179f)) { Fail(cmd, "fov must be in (1, 179) degrees"); yield break; }
            var cam = lockCamera != null ? lockCamera : Camera.main;
            if (cam == null) { Fail(cmd, "no Camera.main"); yield break; }
            if (lockCamera == null)
            {
                lockCamera = cam;
                lockBrain = cam.GetComponent<CinemachineBrain>();
                if (lockBrain == null) lockBrain = FindObjectOfType<CinemachineBrain>();
                if (lockBrain != null) lockBrain.enabled = false;
                savedClearFlags = cam.clearFlags;
                savedBackground = cam.backgroundColor;
            }
            lockPosition = pos;
            lockRotation = Quaternion.Euler(cmd.pitch, cmd.yaw, 0f);
            lockFov = cmd.fov;
            if (cmd.show_player && !showPlayer) ShowPlayer();
            showPlayer = cmd.show_player;
            if (cmd.keep_post && !keepPost) RestorePost();
            keepPost = cmd.keep_post;
            flatSky = cmd.flat_sky;
            HoldLock();
            var until = Time.unscaledTime + Math.Max(cmd.settle_seconds, 0f);
            for (var i = 0; i < cmd.settle_frames || Time.unscaledTime < until; i++) yield return null;
            yield return new WaitForEndOfFrame();
            Succeed(cmd, Pose(cmd));
        }

        private IEnumerator DoShot(Command cmd)
        {
            if (string.IsNullOrEmpty(cmd.path)) { Fail(cmd, "shot needs path"); yield break; }
            yield return new WaitForEndOfFrame();
            var reply = Pose(cmd);
            var tex = ScreenCapture.CaptureScreenshotAsTexture();
            try
            {
                var dir = Path.GetDirectoryName(cmd.path);
                if (!string.IsNullOrEmpty(dir)) Directory.CreateDirectory(dir);
                var tmp = cmd.path + ".tmp";
                File.WriteAllBytes(tmp, tex.EncodeToPNG());
                ReplaceFile(tmp, cmd.path);
            }
            finally
            {
                Destroy(tex);
            }
            reply.path = cmd.path;
            Succeed(cmd, reply);
        }

        private IEnumerator DoReadback(Command cmd)
        {
            yield return new WaitForEndOfFrame();
            Succeed(cmd, Pose(cmd));
        }

        private IEnumerator DoRelease(Command cmd)
        {
            ReleaseLock();
            yield return null;
            Succeed(cmd, Pose(cmd));
        }

        private IEnumerator DoQuit(Command cmd)
        {
            ReleaseLock();
            Succeed(cmd, Pose(cmd));
            yield return null;
            Application.Quit();
        }

        // ---- the camera lock -------------------------------------------------------------------------------------

        private void ApplyPose()
        {
            var t = lockCamera.transform;
            t.SetPositionAndRotation(lockPosition, lockRotation);
            lockCamera.fieldOfView = lockFov;
            // A flat sky has no edges: image matching against another game's sky (other clouds, other colours).
            lockCamera.clearFlags = flatSky ? CameraClearFlags.SolidColor : savedClearFlags;
            lockCamera.backgroundColor = flatSky ? Color.black : savedBackground;
        }

        private void HoldLock()
        {
            if (lockCamera == null) return;
            if (lockBrain != null && lockBrain.enabled) lockBrain.enabled = false;
            ApplyPose();
            HideUi();
            HidePlayer();
            if (!keepPost) PostOff();
        }

        // ReactiveGameplayPostProcessing swaps in a fresh profile copy when a menu opens or closes, so this runs every
        // frame; it only touches the volumes' runtime profile instances, never the profile assets.
        private void PostOff()
        {
            foreach (var v in FindObjectsOfType<PostProcessVolume>())
            {
                if (!v.enabled || !v.HasInstantiatedProfile() && v.sharedProfile == null) continue;
                foreach (var s in v.profile.settings)
                {
                    if (s == null || !s.active || !PostEffectsOff.Contains(s.GetType())) continue;
                    s.active = false;
                    postOff.Add(s);
                }
            }
        }

        private void RestorePost()
        {
            foreach (var s in postOff) if (s != null) s.active = true;
            postOff.Clear();
        }

        private void ShowPlayer()
        {
            foreach (var r in hiddenRenderers) if (r != null) r.enabled = true;
            hiddenRenderers.Clear();
            playerRoots.Clear();
        }

        // Screen-space canvases are the HUD, menus and the console; world-space ones belong to the level and stay.
        private void HideUi()
        {
            foreach (var c in FindObjectsOfType<Canvas>())
            {
                if (!c.enabled || !c.isRootCanvas || c.renderMode == RenderMode.WorldSpace) continue;
                c.enabled = false;
                hiddenCanvases.Add(c);
            }
        }

        private void HidePlayer()
        {
            if (showPlayer) return;
            playerRoots.Clear();
            var board = ImportantStuff.PlayerSkateboard;
            if (board == null) return;
            var roots = new List<Transform>();
            var body = board.RigidBody;
            if (body != null) roots.Add(body.transform.root);
            if (board.Rider != null) roots.Add(board.Rider.transform.root);
            foreach (var root in roots.Distinct())
            {
                var renderers = root.GetComponentsInChildren<Renderer>(true);
                if (renderers.Length > MaxPlayerRenderers)
                {
                    playerRoots.Add(root.name + " (skipped: " + renderers.Length + " renderers)");
                    continue;
                }
                playerRoots.Add(root.name);
                foreach (var r in renderers)
                {
                    if (!r.enabled) continue;
                    r.enabled = false;
                    hiddenRenderers.Add(r);
                }
            }
        }

        private void ReleaseLock()
        {
            if (lockCamera == null) return;
            if (lockBrain != null) lockBrain.enabled = true;
            lockCamera.clearFlags = savedClearFlags;
            lockCamera.backgroundColor = savedBackground;
            flatSky = false;
            foreach (var c in hiddenCanvases) if (c != null) c.enabled = true;
            hiddenCanvases.Clear();
            ShowPlayer();
            RestorePost();
            showPlayer = false;
            keepPost = false;
            lockCamera = null;
            lockBrain = null;
            Logger.LogInfo("CAMERA RELEASED");
        }

        // ---- replies ---------------------------------------------------------------------------------------------

        private Reply Pose(Command cmd)
        {
            var r = new Reply
            {
                seq = cmd.seq,
                op = cmd.op,
                scene = SceneManager.GetActiveScene().name,
                time = Time.unscaledTime,
                frame = Time.frameCount,
                locked = lockCamera != null,
                screen_width = Screen.width,
                screen_height = Screen.height,
                hidden_canvases = hiddenCanvases.Count(c => c != null && !c.enabled),
                hidden_renderers = hiddenRenderers.Count(x => x != null && !x.enabled),
                player_roots = playerRoots.ToArray(),
            };
            try { r.gameplay_obstructed = UIManager.IsGameplayObstructed(); } catch (Exception) { }
            var cam = lockCamera != null ? lockCamera : Camera.main;
            if (cam != null)
            {
                var t = cam.transform;
                var f = t.forward;
                r.cam_name = cam.gameObject.name;
                r.cam_position = Vec(t.position);
                r.cam_euler = Vec(t.eulerAngles);
                r.cam_forward = Vec(f);
                r.cam_bearing = Mathf.Atan2(f.x, f.z) * Mathf.Rad2Deg;
                r.cam_fov = cam.fieldOfView;
                r.cam_fov_projection = 2f * Mathf.Atan(1f / cam.projectionMatrix[1, 1]) * Mathf.Rad2Deg;
                r.cam_near = cam.nearClipPlane;
                r.cam_far = cam.farClipPlane;
                r.other_cameras = Camera.allCameras.Where(c => c != cam).Select(c => Format(
                    "{0} depth {1} target {2} mask 0x{3:X}", c.gameObject.name, c.depth,
                    c.targetTexture != null ? c.targetTexture.name : "screen", c.cullingMask)).ToArray();
            }
            r.kept_canvases = FindObjectsOfType<Canvas>().Where(c => c.enabled && c.isRootCanvas)
                .Select(c => c.gameObject.name + " " + c.renderMode).ToArray();
            r.post_effects_off = postOff.Where(s => s != null && !s.active).Select(s => s.GetType().Name).Distinct()
                .ToArray();
            r.post_effects_on = FindObjectsOfType<PostProcessVolume>().Where(v => v.enabled)
                .SelectMany(v => (v.HasInstantiatedProfile() ? v.profile : v.sharedProfile)?.settings
                                 ?? new List<PostProcessEffectSettings>())
                .Where(s => s != null && s.active).Select(s => s.GetType().Name).Distinct().ToArray();
            if (cam != null)
            {
                // Is the camera in or against level geometry? Non-trigger colliders within CamProbe of it ("name
                // (root)"), and the distance down to the first collider below (-1: none within 1000 m). The player's,
                // the camera's own and the virtual cameras' follow/look-at targets' colliders don't count: at a spawn
                // the game's "Camera Follow" target sits right under the camera.
                var at = cam.transform.position;
                var own = OwnTransforms(cam);
                r.cam_overlaps = Physics.OverlapSphere(at, CamProbe, ~0, QueryTriggerInteraction.Ignore)
                    .Where(c => !IsUnder(c.transform, own))
                    .Select(c => c.gameObject.name + " (" + c.transform.root.name + ")").Distinct().ToArray();
                var below = Physics.RaycastAll(at, Vector3.down, 1000f, ~0, QueryTriggerInteraction.Ignore)
                    .Where(h => !IsUnder(h.collider.transform, own)).OrderBy(h => h.distance).ToArray();
                r.cam_ground = below.Length > 0 ? below[0].distance : -1f;
            }
            var board = ImportantStuff.PlayerSkateboard;
            if (board != null && board.RigidBody != null)
            {
                r.player_position = Vec(board.RigidBody.position);
                r.player_yaw = board.RigidBody.rotation.eulerAngles.y;
            }
            return r;
        }

        private void Succeed(Command cmd, Reply r)
        {
            r.ok = true;
            WriteReply(r);
        }

        private void Fail(Command cmd, string error)
        {
            Reply r;
            try { r = Pose(cmd); } catch (Exception) { r = new Reply { seq = cmd.seq, op = cmd.op }; }
            r.ok = false;
            r.error = error;
            Logger.LogError("OP " + cmd.op + " FAILED: " + error);
            WriteReply(r);
        }

        private void WriteReply(Reply r)
        {
            var json = JsonUtility.ToJson(r);
            Logger.LogInfo("REPLY " + json);
            var tmp = replyPath + ".tmp";
            File.WriteAllText(tmp, json);
            ReplaceFile(tmp, replyPath);
        }

        // The runner may have the old file open for reading (no delete sharing on Windows): retry for up to 2 s.
        private static void ReplaceFile(string tmp, string dest)
        {
            for (var attempt = 0; ; attempt++)
            {
                try
                {
                    if (File.Exists(dest)) File.Delete(dest);
                    File.Move(tmp, dest);
                    return;
                }
                catch (IOException) when (attempt < 200)
                {
                    System.Threading.Thread.Sleep(10);
                }
            }
        }

        // ---- helpers ---------------------------------------------------------------------------------------------

        // The player's roots, the camera's root and every active virtual camera's Follow and LookAt target.
        private static List<Transform> OwnTransforms(Camera cam)
        {
            var own = new List<Transform> { cam.transform.root };
            var board = ImportantStuff.PlayerSkateboard;
            if (board != null && board.RigidBody != null) own.Add(board.RigidBody.transform.root);
            if (board != null && board.Rider != null) own.Add(board.Rider.transform.root);
            foreach (var v in FindObjectsOfType<CinemachineVirtualCameraBase>())
            {
                if (v.Follow != null) own.Add(v.Follow);
                if (v.LookAt != null) own.Add(v.LookAt);
            }
            return own;
        }

        private static bool IsUnder(Transform t, List<Transform> roots)
        {
            return roots.Any(r => t == r || t.IsChildOf(r));
        }

        private static bool TryVector(float[] v, out Vector3 result)
        {
            result = Vector3.zero;
            if (v == null || v.Length != 3) return false;
            result = new Vector3(v[0], v[1], v[2]);
            return true;
        }

        private static float[] Vec(Vector3 v)
        {
            return new[] { v.x, v.y, v.z };
        }

        private static void TryDelete(string path)
        {
            try { if (File.Exists(path)) File.Delete(path); } catch (IOException) { }
        }

        private static string Format(string format, params object[] args)
        {
            return string.Format(CultureInfo.InvariantCulture, format, args);
        }
    }
}
