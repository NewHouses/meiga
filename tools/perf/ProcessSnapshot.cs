// Reads snapshots of every process and thread in the system with
// NtQuerySystemInformation(SystemProcessInformation), the same source that
// Task Manager and Process Explorer use, and adds up what each group of
// processes uses from one snapshot to the next. One snapshot takes a few
// milliseconds, needs no process handles and works without administrator
// rights, so it can also read processes such as dwm.exe, which OpenProcess
// cannot open.
//
// This work is done in C# and not in PowerShell so that the measurement itself
// uses almost no CPU. measure.ps1 loads this file with Add-Type.
//
// The structure offsets are for 64-bit Windows; measure.ps1 refuses to run in
// a 32-bit process.

using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Runtime.InteropServices;

namespace Meiga.Perf
{
    public sealed class ProcessSample
    {
        public int Id;
        public int ParentId;
        public string Name = "";
        public int SessionId;
        /// <summary>Creation time (FILETIME, UTC).</summary>
        public long CreateTime;
        /// <summary>Identity of the process: its id and its creation time, because Windows reuses ids.</summary>
        public string Key = "";
        /// <summary>CPU time in user mode, in 100 ns units.</summary>
        public long UserTime;
        /// <summary>CPU time in kernel mode, in 100 ns units.</summary>
        public long KernelTime;
        /// <summary>Processor cycles used by all the threads of the process.</summary>
        public ulong CycleTime;
        /// <summary>Committed private memory ("Private Bytes", "Commit size"), in bytes.</summary>
        public long PrivateBytes;
        public long WorkingSet;
        public long PrivateWorkingSet;
        public int ThreadCount;
        public int HandleCount;
        /// <summary>Context switches of each thread, by thread id.</summary>
        public Dictionary<long, uint> ContextSwitches = new Dictionary<long, uint>();

        /// <summary>
        /// Context switches since an earlier sample of the same process, or since the
        /// process started if there is none. A thread that started in between counts
        /// from zero; the switches of a thread that exited in between are lost.
        /// </summary>
        public long SwitchesSince(ProcessSample earlier)
        {
            long total = 0;
            foreach (KeyValuePair<long, uint> thread in ContextSwitches)
            {
                uint start;
                // A smaller count means that a new thread got the id of an exited one.
                if (earlier != null && earlier.ContextSwitches.TryGetValue(thread.Key, out start) && thread.Value >= start)
                {
                    total += thread.Value - start;
                }
                else
                {
                    total += thread.Value;
                }
            }
            return total;
        }
    }

    public sealed class Snapshot
    {
        public Dictionary<int, ProcessSample> Processes = new Dictionary<int, ProcessSample>();
        /// <summary>Stopwatch timestamp of the moment of the snapshot.</summary>
        public long Timestamp;

        public double SecondsSince(Snapshot earlier)
        {
            return (Timestamp - earlier.Timestamp) / (double)Stopwatch.Frequency;
        }

        /// <summary>
        /// Context switches of every process since an earlier snapshot. The idle
        /// process is left out: its threads only run when there is nothing to do.
        /// </summary>
        public long SwitchesSince(Snapshot earlier)
        {
            long total = 0;
            foreach (ProcessSample process in Processes.Values)
            {
                if (process.Id == 0)
                {
                    continue;
                }
                ProcessSample before;
                if (!earlier.Processes.TryGetValue(process.Id, out before) || before.CreateTime != process.CreateTime)
                {
                    before = null;
                }
                total += process.SwitchesSince(before);
            }
            return total;
        }
    }

    public static class ProcessSnapshot
    {
        private const int SystemProcessInformation = 5;
        private const int StatusInfoLengthMismatch = unchecked((int)0xC0000004);
        private const ushort AllProcessorGroups = 0xFFFF;
        private const int SmRemoteSession = 0x1000;

        // SYSTEM_PROCESS_INFORMATION (x64).
        private const int NextEntryOffset = 0x00;
        private const int NumberOfThreads = 0x04;
        private const int WorkingSetPrivateSize = 0x08;
        private const int CycleTimeOffset = 0x18;
        private const int CreateTimeOffset = 0x20;
        private const int UserTimeOffset = 0x28;
        private const int KernelTimeOffset = 0x30;
        private const int ImageNameLength = 0x38;
        private const int ImageNameBuffer = 0x40;
        private const int UniqueProcessId = 0x50;
        private const int InheritedFromUniqueProcessId = 0x58;
        private const int HandleCountOffset = 0x60;
        private const int SessionIdOffset = 0x64;
        private const int WorkingSetSize = 0x90;
        private const int PrivatePageCount = 0xC8;
        private const int ProcessInfoSize = 0x100;

        // SYSTEM_THREAD_INFORMATION (x64), right after each process entry.
        private const int ThreadInfoSize = 0x50;
        private const int UniqueThread = 0x30;
        private const int ContextSwitchesOffset = 0x40;

        [DllImport("ntdll.dll")]
        private static extern int NtQuerySystemInformation(
            int systemInformationClass, IntPtr systemInformation, int length, out int returnLength);

        [DllImport("kernel32.dll")]
        private static extern int GetActiveProcessorCount(ushort groupNumber);

        [DllImport("user32.dll")]
        private static extern int GetSystemMetrics(int index);

        /// <summary>Logical processors of the whole machine, in every processor group.</summary>
        public static int ProcessorCount
        {
            get { return GetActiveProcessorCount(AllProcessorGroups); }
        }

        /// <summary>Whether this session is a Remote Desktop session.</summary>
        public static bool IsRemoteSession
        {
            get { return GetSystemMetrics(SmRemoteSession) != 0; }
        }

        public static Snapshot Take()
        {
            int length = 4 * 1024 * 1024;
            while (true)
            {
                IntPtr buffer = Marshal.AllocHGlobal(length);
                try
                {
                    int needed;
                    long before = Stopwatch.GetTimestamp();
                    int status = NtQuerySystemInformation(SystemProcessInformation, buffer, length, out needed);
                    long after = Stopwatch.GetTimestamp();
                    if (status == StatusInfoLengthMismatch)
                    {
                        // Processes can start before the next call: leave some margin.
                        length = Math.Max(length * 2, needed + 1024 * 1024);
                        continue;
                    }
                    if (status != 0)
                    {
                        throw new InvalidOperationException(
                            "NtQuerySystemInformation failed with NTSTATUS 0x" + status.ToString("X8"));
                    }
                    Snapshot snapshot = Parse(buffer);
                    snapshot.Timestamp = before + (after - before) / 2;
                    return snapshot;
                }
                finally
                {
                    Marshal.FreeHGlobal(buffer);
                }
            }
        }

        private static Snapshot Parse(IntPtr buffer)
        {
            var snapshot = new Snapshot();
            IntPtr entry = buffer;
            while (true)
            {
                int threads = Marshal.ReadInt32(entry, NumberOfThreads);
                int nameBytes = (ushort)Marshal.ReadInt16(entry, ImageNameLength);
                IntPtr name = Marshal.ReadIntPtr(entry, ImageNameBuffer);

                var sample = new ProcessSample
                {
                    Id = (int)Marshal.ReadInt64(entry, UniqueProcessId),
                    ParentId = (int)Marshal.ReadInt64(entry, InheritedFromUniqueProcessId),
                    // The idle process has no image name.
                    Name = name == IntPtr.Zero ? "Idle" : (Marshal.PtrToStringUni(name, nameBytes / 2) ?? ""),
                    SessionId = Marshal.ReadInt32(entry, SessionIdOffset),
                    CreateTime = Marshal.ReadInt64(entry, CreateTimeOffset),
                    UserTime = Marshal.ReadInt64(entry, UserTimeOffset),
                    KernelTime = Marshal.ReadInt64(entry, KernelTimeOffset),
                    CycleTime = (ulong)Marshal.ReadInt64(entry, CycleTimeOffset),
                    PrivateBytes = Marshal.ReadInt64(entry, PrivatePageCount),
                    WorkingSet = Marshal.ReadInt64(entry, WorkingSetSize),
                    PrivateWorkingSet = Marshal.ReadInt64(entry, WorkingSetPrivateSize),
                    ThreadCount = threads,
                    HandleCount = Marshal.ReadInt32(entry, HandleCountOffset),
                };
                sample.Key = sample.Id + ":" + sample.CreateTime;

                for (int i = 0; i < threads; i++)
                {
                    IntPtr thread = IntPtr.Add(entry, ProcessInfoSize + i * ThreadInfoSize);
                    long threadId = Marshal.ReadInt64(thread, UniqueThread);
                    sample.ContextSwitches[threadId] = unchecked((uint)Marshal.ReadInt32(thread, ContextSwitchesOffset));
                }

                snapshot.Processes[sample.Id] = sample;

                int next = Marshal.ReadInt32(entry, NextEntryOffset);
                if (next == 0)
                {
                    break;
                }
                entry = IntPtr.Add(entry, next);
            }
            return snapshot;
        }
    }

    /// <summary>Memory and threads of a process or of a group, over the snapshots.</summary>
    public sealed class MemoryStat
    {
        public int Samples;
        public double PrivateSum;
        public double PrivateMax;
        public double WorkingSetSum;
        public double ThreadsSum;
        public double ThreadsMax;

        public void Add(double privateBytes, double workingSet, double threads)
        {
            Samples++;
            PrivateSum += privateBytes;
            PrivateMax = Math.Max(PrivateMax, privateBytes);
            WorkingSetSum += workingSet;
            ThreadsSum += threads;
            ThreadsMax = Math.Max(ThreadsMax, threads);
        }
    }

    /// <summary>A process of a group, followed from snapshot to snapshot.</summary>
    public sealed class TrackedProcess
    {
        public string Group = "";
        /// <summary>Sample at the start of the window, or null if the process started later.</summary>
        public ProcessSample Start;
        /// <summary>Latest sample in which the process appeared.</summary>
        public ProcessSample Last;
        /// <summary>Number of the snapshot of Last.</summary>
        public int LastIndex;
        /// <summary>Context switches inside the window.</summary>
        public long Switches;
        public MemoryStat Memory = new MemoryStat();
    }

    /// <summary>
    /// Sorts the processes of each snapshot into groups and adds up what they use.
    /// The terminal groups grow from the processes given to AddTerminal with their
    /// descendants; the system groups are found by name.
    /// </summary>
    public sealed class Tracker
    {
        public static readonly string[] GroupNames =
        {
            "terminal", "console", "shells", "dwm", "termservice", "wudfhost", "harness", "system"
        };

        private static readonly HashSet<string> ConsoleHosts =
            new HashSet<string>(StringComparer.OrdinalIgnoreCase) { "OpenConsole.exe", "conhost.exe" };

        private readonly HashSet<string> terminalNames = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
        private readonly Dictionary<string, string> groups = new Dictionary<string, string>();
        private readonly int sessionId;
        private readonly int termServiceId;

        /// <summary>Every process that has been in a group, by key.</summary>
        public readonly Dictionary<string, TrackedProcess> Processes = new Dictionary<string, TrackedProcess>();
        /// <summary>Memory and threads of each group; "system" is every process except this script.</summary>
        public readonly Dictionary<string, MemoryStat> GroupMemory = new Dictionary<string, MemoryStat>();
        public Snapshot First;
        public Snapshot Last;
        /// <summary>Number of the last snapshot; the first one is 0.</summary>
        public int Index = -1;
        /// <summary>Context switches of every process except the idle one, this script included.</summary>
        public long SystemSwitches;

        /// <param name="sessionId">Session whose dwm.exe is measured.</param>
        /// <param name="termServiceId">Process id of the Remote Desktop service, or 0.</param>
        public Tracker(int sessionId, int termServiceId)
        {
            this.sessionId = sessionId;
            this.termServiceId = termServiceId;
        }

        public void AddTerminal(ProcessSample process)
        {
            groups[process.Key] = "terminal";
            terminalNames.Add(process.Name);
        }

        public void AddHarness(ProcessSample process)
        {
            groups[process.Key] = "harness";
        }

        public void Add(Snapshot snapshot)
        {
            Index++;
            if (First == null)
            {
                First = snapshot;
            }
            Classify(snapshot);

            int count = GroupNames.Length;
            int system = count - 1;
            int harness = Array.IndexOf(GroupNames, "harness");
            var processes = new int[count];
            var privateBytes = new double[count];
            var workingSet = new double[count];
            var threads = new double[count];
            foreach (ProcessSample p in snapshot.Processes.Values)
            {
                if (p.Id == 0)
                {
                    continue;
                }
                string group;
                int g = groups.TryGetValue(p.Key, out group) ? Array.IndexOf(GroupNames, group) : -1;
                // "system" is the whole machine without this script.
                int[] targets = g < 0 ? new[] { system } : g == harness ? new[] { g } : new[] { system, g };
                foreach (int target in targets)
                {
                    processes[target]++;
                    privateBytes[target] += p.PrivateBytes;
                    workingSet[target] += p.WorkingSet;
                    threads[target] += p.ThreadCount;
                }
                if (g < 0)
                {
                    continue;
                }

                TrackedProcess tracked;
                if (!Processes.TryGetValue(p.Key, out tracked))
                {
                    // What it uses counts from the start of the window, or from zero if
                    // it started later.
                    ProcessSample start;
                    if (!First.Processes.TryGetValue(p.Id, out start) || start.CreateTime != p.CreateTime)
                    {
                        start = null;
                    }
                    tracked = new TrackedProcess { Group = group, Start = start, Last = start };
                    Processes[p.Key] = tracked;
                }
                tracked.Switches += p.SwitchesSince(tracked.Last);
                tracked.Last = p;
                tracked.LastIndex = Index;
                tracked.Memory.Add(p.PrivateBytes, p.WorkingSet, p.ThreadCount);
            }

            for (int g = 0; g < count; g++)
            {
                if (processes[g] == 0)
                {
                    continue;
                }
                MemoryStat stat;
                if (!GroupMemory.TryGetValue(GroupNames[g], out stat))
                {
                    stat = new MemoryStat();
                    GroupMemory[GroupNames[g]] = stat;
                }
                stat.Add(privateBytes[g], workingSet[g], threads[g]);
            }

            if (Last != null)
            {
                SystemSwitches += snapshot.SwitchesSince(Last);
            }
            Last = snapshot;
        }

        private void Classify(Snapshot snapshot)
        {
            // The descendants of the terminal: from the processes already in the
            // terminal groups down to their new children.
            var children = new Dictionary<int, List<ProcessSample>>();
            var queue = new Queue<ProcessSample>();
            foreach (ProcessSample p in snapshot.Processes.Values)
            {
                List<ProcessSample> list;
                if (!children.TryGetValue(p.ParentId, out list))
                {
                    list = new List<ProcessSample>();
                    children[p.ParentId] = list;
                }
                list.Add(p);
                string group;
                if (groups.TryGetValue(p.Key, out group) && IsTerminalGroup(group))
                {
                    queue.Enqueue(p);
                }
            }
            while (queue.Count > 0)
            {
                ProcessSample parent = queue.Dequeue();
                List<ProcessSample> list;
                if (!children.TryGetValue(parent.Id, out list))
                {
                    continue;
                }
                string parentGroup = groups[parent.Key];
                foreach (ProcessSample child in list)
                {
                    // A parent always starts before its children; otherwise the id was reused.
                    if (groups.ContainsKey(child.Key) || child.CreateTime < parent.CreateTime)
                    {
                        continue;
                    }
                    string group = "shells";
                    if (parentGroup == "terminal")
                    {
                        if (terminalNames.Contains(child.Name))
                        {
                            group = "terminal";
                        }
                        else if (ConsoleHosts.Contains(child.Name))
                        {
                            group = "console";
                        }
                    }
                    groups[child.Key] = group;
                    queue.Enqueue(child);
                }
            }

            // The processes that put the windows on the screen.
            foreach (ProcessSample p in snapshot.Processes.Values)
            {
                if (groups.ContainsKey(p.Key))
                {
                    continue;
                }
                if (string.Equals(p.Name, "dwm.exe", StringComparison.OrdinalIgnoreCase) && p.SessionId == sessionId)
                {
                    groups[p.Key] = "dwm";
                }
                else if (termServiceId != 0 && p.Id == termServiceId)
                {
                    groups[p.Key] = "termservice";
                }
                else if (string.Equals(p.Name, "WUDFHost.exe", StringComparison.OrdinalIgnoreCase))
                {
                    groups[p.Key] = "wudfhost";
                }
            }
        }

        private static bool IsTerminalGroup(string group)
        {
            return group == "terminal" || group == "console" || group == "shells";
        }
    }
}
