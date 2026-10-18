// SkateBIRD teleport probe for the lab copy.
//
// Watches C:/Users/kalmarv/ds-modding/data/skatebird-teleport.json:
//   { "x": -18.7, "y": 1.189, "z": 84.8, "yaw": 172.9341 }
// teleports the live skater to that Unity position/yaw (Quaternion.Euler(0, yaw, 0)),
// zeroes the board Rigidbody for six fixed frames, then logs the read-back pose.
// Optional cam_yaw/cam_pitch: locks Camera.main to that rotation for six seconds (CinemachineBrain
// disabled while locked) and logs the measured forward bearing.
using System;
using System.Collections;
using System.Globalization;
using System.IO;
using System.Text.RegularExpressions;
using BepInEx;
using Cinemachine;
using Skatebirb;
using Skatebirb.Managers;
using Skatebirb.Util;
using UnityEngine;
using UnityEngine.SceneManagement;

namespace SkateBIRDTeleport
{
    [BepInPlugin("skatebird.teleport", "SkateBIRD Teleport", "0.4.0")]
    public class TeleportPlugin : BaseUnityPlugin
    {
        private const string TeleportPath = @"C:/Users/kalmarv/ds-modding/data/skatebird-teleport.json";
        private const string ScenePath = @"C:/Users/kalmarv/ds-modding/data/skatebird-load-scene.json";
        private const float PollSeconds = 0.2f;

        private static readonly Regex Number = new Regex(
            "\"(?<key>x|y|z|yaw)\"\\s*:\\s*(?<value>-?[0-9]+(?:\\.[0-9]+)?(?:[eE][+-]?[0-9]+)?)",
            RegexOptions.Compiled);

        private static readonly Regex CamNumber = new Regex(
            "\"(?<key>cam_yaw|cam_pitch)\"\\s*:\\s*(?<value>-?[0-9]+(?:\\.[0-9]+)?(?:[eE][+-]?[0-9]+)?)",
            RegexOptions.Compiled);

        private static readonly Regex SceneName = new Regex(
            "\"scene\"\\s*:\\s*\"(?<name>[^\"]+)\"", RegexOptions.Compiled);

        private float nextPoll;
        private string lastText;
        private bool waitingLogged;
        private string lastSceneText;
        private string pendingScene;
        private string requestedScene;
        private float cameraLogUntil;
        private float nextCameraLog;
        private Camera lockCamera;
        private CinemachineBrain lockBrain;
        private bool lockBrainWasEnabled;
        private float lockUntil;
        private float lockCheckAt;
        private bool lockCheckLogged;
        private double lockYaw;
        private double lockPitch;
        private Vector3 lockTarget;
        private float lockDistance;
        private float lockTargetY;

        private void Update()
        {
            ServiceCameraLock();
            if (Time.unscaledTime < nextPoll) return;
            nextPoll = Time.unscaledTime + PollSeconds;
            PollScene();
            PollTeleport();
            if (Time.unscaledTime < cameraLogUntil && Time.unscaledTime >= nextCameraLog)
            {
                LogCamera();
                nextCameraLog = Time.unscaledTime + 0.25f;
            }
        }

        private void LateUpdate()
        {
            if (lockCamera != null && Time.unscaledTime < lockUntil) ApplyCameraLock();
        }

        private void PollScene()
        {
            if (requestedScene != null && SceneManager.GetActiveScene().name == requestedScene)
            {
                Logger.LogInfo("SCENE LOADED " + requestedScene);
                requestedScene = null;
            }

            string text;
            try
            {
                if (!File.Exists(ScenePath)) return;
                text = File.ReadAllText(ScenePath);
            }
            catch (IOException)
            {
                return;
            }
            if (text != lastSceneText)
            {
                lastSceneText = text;
                var m = SceneName.Match(text);
                if (m.Success)
                {
                    pendingScene = m.Groups["name"].Value;
                }
                else
                {
                    Logger.LogError("SCENE JSON BAD: expected {\"scene\":\"flashback\"}");
                }
            }
            if (pendingScene != null && !LoadingManager.IsLoading)
            {
                requestedScene = pendingScene;
                pendingScene = null;
                Logger.LogInfo("SCENE LOAD REQUEST " + requestedScene);
                LoadingManager.LoadScene(requestedScene);
            }
        }

        private void PollTeleport()
        {
            string text;
            try
            {
                if (!File.Exists(TeleportPath)) return;
                text = File.ReadAllText(TeleportPath);
            }
            catch (IOException)
            {
                return;
            }
            if (text == lastText) return;
            lastText = text;

            double x, y, z, yaw;
            if (!TryParse(text, out x, out y, out z, out yaw))
            {
                Logger.LogError("TELEPORT JSON BAD: expected {\"x\":..,\"y\":..,\"z\":..,\"yaw\":..}");
                return;
            }

            var board = ImportantStuff.PlayerSkateboard;
            if (board == null)
            {
                if (!waitingLogged)
                {
                    Logger.LogInfo("TELEPORT WAIT: no player yet (ImportantStuff.PlayerSkateboard is null)");
                    waitingLogged = true;
                }
                return;
            }
            waitingLogged = false;
            double? camYaw = null;
            var camPitch = 12.0;
            foreach (Match m in CamNumber.Matches(text))
            {
                double v;
                if (!double.TryParse(m.Groups["value"].Value, NumberStyles.Float,
                        CultureInfo.InvariantCulture, out v))
                    continue;
                if (m.Groups["key"].Value == "cam_yaw") camYaw = v;
                else camPitch = v;
            }
            StartCoroutine(Apply(board, x, y, z, yaw, camYaw, camPitch));
        }

        private IEnumerator Apply(ISkateboardGateway board, double x, double y, double z, double yaw,
            double? camYaw, double camPitch)
        {
            var pos = new Vector3((float)x, (float)y, (float)z);
            var rot = Quaternion.Euler(0f, (float)yaw, 0f);
            Logger.LogInfo(Format("TELEPORT REQUEST ({0:F3},{1:F3},{2:F3} yaw {3:F3})", x, y, z, yaw));

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

            var body = board.RigidBody;
            var p = body.position;
            var e = body.rotation.eulerAngles;
            Logger.LogInfo(Format("TELEPORT OK ({0:F3},{1:F3},{2:F3} yaw {3:F3})", p.x, p.y, p.z, e.y));
            Logger.LogInfo(Format("PLAYER POS ({0:F3},{1:F3},{2:F3}) ROT ({3:F3},{4:F3},{5:F3})",
                p.x, p.y, p.z, e.x, e.y, e.z));
            Logger.LogInfo(Format("PLAYER OBJECT {0} ACTIVE {1}", body.gameObject.name, body.gameObject.activeInHierarchy));
            LogCamera();
            cameraLogUntil = Time.unscaledTime + 1.5f;
            nextCameraLog = 0f;
            if (camYaw.HasValue) BeginCameraLock(x, y, z, camYaw.Value, camPitch);
        }

        private void LogCamera()
        {
            var cam = Camera.main;
            if (cam == null)
            {
                Logger.LogInfo("CAMERA NONE (no Camera.main)");
                return;
            }
            var t = cam.transform;
            var p = t.position;
            var f = t.forward;
            var bearing = Mathf.Atan2(f.x, f.z) * Mathf.Rad2Deg;
            Logger.LogInfo(Format(
                "CAMERA POS ({0:F3},{1:F3},{2:F3}) FWD ({3:F4},{4:F4},{5:F4}) BEARING {6:F2}",
                p.x, p.y, p.z, f.x, f.y, f.z, bearing));
            Logger.LogInfo(Format("CAMERA OBJECT {0} EULER ({1:F2},{2:F2},{3:F2})",
                cam.gameObject.name, t.eulerAngles.x, t.eulerAngles.y, t.eulerAngles.z));
        }

        private void BeginCameraLock(double x, double y, double z, double yaw, double pitch)
        {
            lockCamera = Camera.main;
            if (lockCamera == null)
            {
                Logger.LogError("CAMERA LOCK FAIL: no Camera.main");
                return;
            }
            lockYaw = yaw;
            lockPitch = pitch;
            lockTarget = new Vector3((float)x, (float)y, (float)z);
            lockDistance = 3.0f;
            lockTargetY = 0.7f;
            lockBrain = lockCamera.GetComponent<CinemachineBrain>();
            if (lockBrain == null) lockBrain = FindObjectOfType<CinemachineBrain>();
            if (lockBrain != null)
            {
                lockBrainWasEnabled = lockBrain.enabled;
                lockBrain.enabled = false;
            }
            lockUntil = Time.unscaledTime + 6f;
            lockCheckAt = Time.unscaledTime + 1f;
            lockCheckLogged = false;
            Logger.LogInfo(Format(
                "CAMERA LOCK REQUEST yaw {0:F2} pitch {1:F2} dist {2:F2} target_y {3:F2} camera {4}",
                yaw, pitch, lockDistance, lockTargetY, lockCamera.gameObject.name));
            ApplyCameraLock();
        }

        private void ApplyCameraLock()
        {
            if (lockCamera == null) return;
            var yaw = (float)lockYaw * Mathf.Deg2Rad;
            var pitch = (float)lockPitch * Mathf.Deg2Rad;
            var forward = new Vector3(Mathf.Sin(yaw) * Mathf.Cos(pitch), -Mathf.Sin(pitch),
                Mathf.Cos(yaw) * Mathf.Cos(pitch));
            var target = lockTarget + new Vector3(0f, lockTargetY, 0f);
            lockCamera.transform.position = target - forward * lockDistance;
            lockCamera.transform.rotation = Quaternion.Euler((float)lockPitch, (float)lockYaw, 0f);
        }

        private void ServiceCameraLock()
        {
            if (lockCamera == null) return;
            if (Time.unscaledTime >= lockUntil)
            {
                if (lockBrain != null) lockBrain.enabled = lockBrainWasEnabled;
                Logger.LogInfo("CAMERA LOCK RESTORED");
                lockCamera = null;
                lockBrain = null;
                return;
            }
            ApplyCameraLock();
            if (lockCheckLogged || Time.unscaledTime < lockCheckAt) return;
            lockCheckLogged = true;
            var f = lockCamera.transform.forward;
            var bearing = Mathf.Atan2(f.x, f.z) * Mathf.Rad2Deg;
            var diff = Mathf.DeltaAngle((float)lockYaw, bearing);
            var ok = Mathf.Abs(diff) <= 2f;
            Logger.LogInfo(Format(
                "CAMERA LOCK BEARING {0:F2} TARGET {1:F2} DIFF {2:F2} {3}",
                bearing, lockYaw, diff, ok ? "OK" : "FAIL"));
            if (!ok)
            {
                Logger.LogError(Format(
                    "CAMERA LOCK FAIL bearing {0:F2} target {1:F2} diff {2:F2}",
                    bearing, lockYaw, diff));
            }
        }

        private static string Format(string format, params object[] args)
        {
            return string.Format(CultureInfo.InvariantCulture, format, args);
        }

        private static bool TryParse(string text, out double x, out double y, out double z, out double yaw)
        {
            x = y = z = yaw = 0;
            var seen = 0;
            foreach (Match m in Number.Matches(text))
            {
                double v;
                if (!double.TryParse(m.Groups["value"].Value, NumberStyles.Float, CultureInfo.InvariantCulture, out v))
                    return false;
                switch (m.Groups["key"].Value)
                {
                    case "x": x = v; seen |= 1; break;
                    case "y": y = v; seen |= 2; break;
                    case "z": z = v; seen |= 4; break;
                    case "yaw": yaw = v; seen |= 8; break;
                }
            }
            return seen == 15;
        }
    }
}
