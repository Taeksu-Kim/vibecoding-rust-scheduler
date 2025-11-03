use std::collections::HashMap;

pub struct PromptTemplate {
    template: String,
}

impl PromptTemplate {
    pub fn new(template: impl Into<String>) -> Self {
        Self {
            template: template.into(),
        }
    }

    pub fn render(&self, vars: &HashMap<String, String>) -> String {
        let mut result = self.template.clone();
        for (key, value) in vars {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    }

    pub fn schedule_validation() -> Self {
        Self::new(
            r#"Please review this schedule and provide feedback:

{context}

Focus on:
1. **Realism**: Are the estimated durations realistic?
2. **Balance**: Is there enough break time between tasks?
3. **Priority**: Are high-priority tasks scheduled appropriately?
4. **Energy**: Do complex tasks align with peak productivity hours?
5. **Buffer**: Is there buffer time for unexpected delays?

Provide specific suggestions for improvement."#,
        )
    }

    pub fn task_assistant() -> Self {
        Self::new(
            r#"Current work context:

{context}

Question: {question}

Please provide helpful advice considering:
- Current task progress and time remaining
- Overall schedule for today
- Potential impact on upcoming tasks
- Time management best practices"#,
        )
    }

    pub fn optimization() -> Self {
        Self::new(
            r#"Schedule optimization request:

{context}

Current situation:
{situation}

Please analyze the remaining schedule and suggest:
1. **Adjustments**: Which tasks should be rescheduled or reprioritized?
2. **Time Estimates**: Are any estimates unrealistic given current progress?
3. **Recovery Plan**: How to get back on track?
4. **Trade-offs**: What tasks can be deferred if needed?

Provide a concrete action plan."#,
        )
    }

    pub fn focus_advice() -> Self {
        Self::new(
            r#"Current task: {task_title}
Time remaining: {time_remaining} minutes
Estimated duration: {estimated_duration} minutes

I'm having trouble focusing. What should I do?

Consider:
- Pomodoro technique suggestions
- Break recommendations
- Task breakdown ideas
- Distraction management"#,
        )
    }

    pub fn daily_planning() -> Self {
        Self::new(
            r#"Help me plan my day:

Available time: {available_hours} hours
Key objectives: {objectives}

Previous schedule context:
{context}

Please suggest:
1. **Task List**: What tasks to include
2. **Time Allocation**: Realistic time estimates
3. **Ordering**: Best sequence for tasks
4. **Breaks**: When to schedule breaks
5. **Buffer**: Contingency time recommendations"#,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_render() {
        let template = PromptTemplate::new("Hello {name}, you have {count} tasks");
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("count".to_string(), "5".to_string());

        let result = template.render(&vars);
        assert_eq!(result, "Hello Alice, you have 5 tasks");
    }
}
